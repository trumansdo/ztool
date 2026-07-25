//! HTTP/HTTPS forward proxy — tokio-based, production grade.
//! ponytail: manual into_split + read/write + select! relay
//!   avoids copy_bidirectional flush race (#6519) and split+copy deadlock (#322).

use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const DEFAULT_PORT: u16 = 13873;
const TIMEOUT: Duration = Duration::from_secs(30);
const BUF_SIZE: usize = 8 * 1024;

// ── helpers ──────────────────────────────────────────────────────────────

fn parse_host_port(s: &str, fallback: u16) -> (String, u16) {
    if s.starts_with('[') {
        if let Some(end) = s.find(']') {
            let host = &s[1..end];
            let port = s[end + 1..].strip_prefix(':').and_then(|p| p.parse().ok()).unwrap_or(fallback);
            return (host.to_string(), port);
        }
    }
    match s.rfind(':') {
        Some(idx) => (s[..idx].to_string(), s[idx + 1..].parse().unwrap_or(fallback)),
        None => (s.to_string(), fallback),
    }
}

fn parse_http_url(url: &str) -> Option<(String, u16, String)> {
    let rest = url.strip_prefix("http://")?;
    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (host, port) = parse_host_port(host_port, 80);
    Some((host, port, path.to_string()))
}

// ── relay ────────────────────────────────────────────────────────────────

async fn relay_duplex(client: TcpStream, remote: TcpStream) -> io::Result<()> {
    let (mut cr, mut cw) = client.into_split();
    let (mut rr, mut rw) = remote.into_split();

    let mut cbuf = vec![0u8; BUF_SIZE];
    let mut rbuf = vec![0u8; BUF_SIZE];

    loop {
        tokio::select! {
            // client → remote
            result = cr.read(&mut cbuf) => {
                match result? {
                    0 => {
                        rw.shutdown().await?;
                        break;
                    }
                    n => rw.write_all(&cbuf[..n]).await?,
                }
            }
            // remote → client
            result = rr.read(&mut rbuf) => {
                match result? {
                    0 => {
                        cw.shutdown().await?;
                        break;
                    }
                    n => cw.write_all(&rbuf[..n]).await?,
                }
            }
        }
    }

    // drain remaining direction
    let _ = tokio::io::copy(&mut cr, &mut rw).await;
    let _ = tokio::io::copy(&mut rr, &mut cw).await;

    Ok(())
}

// ── handlers ─────────────────────────────────────────────────────────────

async fn handle_connect(client: TcpStream, target: &str) -> io::Result<()> {
    let (host, port) = parse_host_port(target, 443);
    eprintln!("[+] CONNECT {host}:{port}");

    let remote = match TcpStream::connect((host.as_str(), port)).await {
        Ok(s) => s,
        Err(e) => {
            // send back 502 before returning error so tunnel spans get cleaned up
            let (mut client, _remote_err) = (client, e);
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return Err(io::Error::new(io::ErrorKind::ConnectionRefused, "upstream unreachable"));
        }
    };

    let mut client = client;
    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await?;
    client.flush().await?;

    tokio::time::timeout(TIMEOUT, relay_duplex(client, remote)).await?
}

async fn handle_http(
    mut client: TcpStream,
    method: &str,
    url: &str,
    version: &str,
    headers: &[String],
) -> io::Result<()> {
    let (host, port, path) = match parse_http_url(url) {
        Some(p) => p,
        None => {
            let _ = client.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
            return Ok(());
        }
    };

    eprintln!("[+] HTTP {method} {host}:{port}{path}");

    let mut remote = match TcpStream::connect((host.as_str(), port)).await {
        Ok(s) => s,
        Err(e) => {
            let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n").await;
            return Err(e);
        }
    };

    let req_line = format!("{method} {path} {version}\r\n");
    remote.write_all(req_line.as_bytes()).await?;

    for h in &headers[1..] {
        if h.starts_with("Proxy-Connection") || h.starts_with("Proxy-Authorization") {
            continue;
        }
        remote.write_all(h.as_bytes()).await?;
        if !h.ends_with('\n') {
            remote.write_all(b"\r\n").await?;
        }
    }
    remote.write_all(b"\r\n").await?;

    let (mut cr, mut cw) = client.split();
    let (mut rr, mut rw) = remote.split();

    let body = io::copy(&mut cr, &mut rw);
    let resp = io::copy(&mut rr, &mut cw);

    tokio::time::timeout(TIMEOUT, async {
        tokio::select! {
            r = body => { let _ = r; }
            r = resp => { let _ = r; }
        }
        Ok::<_, io::Error>(())
    })
    .await??;

    Ok(())
}

// ── connection handler ───────────────────────────────────────────────────

async fn serve(mut client: TcpStream) -> io::Result<()> {
    let timeout = tokio::time::timeout(TIMEOUT, async {
        let mut buf = vec![0u8; BUF_SIZE];
        let mut data = Vec::new();

        loop {
            let n = client.read(&mut buf).await?;
            if n == 0 {
                return Ok::<Vec<u8>, io::Error>(data);
            }
            data.extend_from_slice(&buf[..n]);
            if data.windows(4).any(|w| w == b"\r\n\r\n") {
                return Ok(data);
            }
            if data.len() > 64 * 1024 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "header too large"));
            }
        }
    })
    .await;

    match timeout {
        Ok(Ok(data)) => {
            if data.is_empty() {
                return Ok(());
            }
            handle_client(client, &data).await
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(io::Error::new(io::ErrorKind::TimedOut, "read timeout")),
    }
}

async fn handle_client(mut client: TcpStream, raw: &[u8]) -> io::Result<()> {
    let text = String::from_utf8_lossy(raw);
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        let _ = client.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
        return Ok(());
    }

    let parts: Vec<&str> = lines[0].trim().split_whitespace().collect();
    if parts.len() < 3 {
        let _ = client.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n").await;
        return Ok(());
    }

    let method = parts[0].to_uppercase();
    let url = parts[1];
    let version = parts[2];

    let headers: Vec<String> = lines[1..]
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string() + "\r\n")
        .collect();

    if method == "CONNECT" {
        handle_connect(client, url).await
    } else {
        handle_http(client, &method, url, version, &headers).await
    }
}

// ── main ─────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> io::Result<()> {
    let port = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    eprintln!("========================================");
    eprintln!(" Rust HTTP/HTTPS Proxy (tokio)");
    eprintln!(" Listening on 0.0.0.0:{port}");
    eprintln!("========================================");

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[-] accept: {e}");
                continue;
            }
        };
        tokio::spawn(async {
            if let Err(e) = serve(stream).await {
                let k = e.kind();
                if k != io::ErrorKind::ConnectionReset
                    && k != io::ErrorKind::BrokenPipe
                    && k != io::ErrorKind::UnexpectedEof
                    && k != io::ErrorKind::TimedOut
                {
                    eprintln!("[-] client error: {e}");
                }
            }
        });
    }
}
