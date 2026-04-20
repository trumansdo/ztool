use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::time::Duration;

fn scan_port(ip: Ipv4Addr, port: u16, timeout: Duration) -> (bool, Duration) {
    let addr = SocketAddr::new(IpAddr::V4(ip), port);
    let start = std::time::Instant::now();
    let is_open = TcpStream::connect_timeout(&addr, timeout).is_ok();
    let elapsed = start.elapsed();
    (is_open, elapsed)
}

fn scan_all_ports(ip: Ipv4Addr) -> Vec<u16> {
    let timeout = Duration::from_millis(50);
    (1..=100)  // 只扫描前100个端口用于测试
        .map(|port| {
            let (is_open, elapsed) = scan_port(ip, port, timeout);
            println!("端口 {:>5} -> {} 耗时 {:?}", port, if is_open { "开放" } else { "关闭" }, elapsed);
            (port, is_open)
        })
        .filter(|(_, is_open)| *is_open)
        .map(|(port, _)| port)
        .collect()
}

#[test]
fn demo() {
    let start = std::time::Instant::now();
    let open_ports = scan_all_ports(Ipv4Addr::new(127, 0, 0, 1));
    let elapsed = start.elapsed();
    
    println!("\n========== 扫描结果 ==========");
    println!("总耗时: {:?}", elapsed);
    println!("开放端口数: {}", open_ports.len());
    println!("开放端口: {:?}", open_ports);
    
    assert!(elapsed.as_secs() < 60, "扫描超时: {:?}", elapsed);
}