use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PacketInfo {
    pub timestamp: String,
    pub src_ip: String,
    pub dst_ip: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: String,
    pub length: usize,
    pub http_info: Option<HttpInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct HttpInfo {
    pub method: Option<String>,
    pub uri: Option<String>,
    pub version: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub response_code: Option<u16>,
}

impl HttpInfo {
    pub fn parse_request(data: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.lines().collect();
        
        if lines.is_empty() {
            return None;
        }
        
        let mut info = HttpInfo::default();
        
        if let Some(request_line) = lines.first() {
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() >= 3 {
                info.method = Some(parts[0].to_string());
                info.uri = Some(parts[1].to_string());
                info.version = Some(parts[2].to_string());
            }
        }
        
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                info.headers.insert(key, value);
            }
        }
        
        if let Some(body_start) = text.find("\r\n\r\n") {
            let body = text[body_start + 4..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        } else if let Some(body_start) = text.find("\n\n") {
            let body = text[body_start + 2..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        }
        
        Some(info)
    }
    
    pub fn parse_response(data: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.lines().collect();
        
        if lines.is_empty() {
            return None;
        }
        
        let mut info = HttpInfo::default();
        
        if let Some(status_line) = lines.first() {
            let parts: Vec<&str> = status_line.split_whitespace().collect();
            if parts.len() >= 2 {
                info.version = Some(parts[0].to_string());
                if let Ok(code) = parts[1].parse::<u16>() {
                    info.response_code = Some(code);
                }
            }
        }
        
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                info.headers.insert(key, value);
            }
        }
        
        if let Some(body_start) = text.find("\r\n\r\n") {
            let body = text[body_start + 4..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        } else if let Some(body_start) = text.find("\n\n") {
            let body = text[body_start + 2..].trim();
            if !body.is_empty() {
                info.body = Some(body.to_string());
            }
        }
        
        Some(info)
    }
    
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref method) = self.method {
            parts.push(format!("{} {} {}", method, self.uri.as_ref().unwrap_or(&String::new()), self.version.as_ref().unwrap_or(&String::new())));
        }
        
        if let Some(code) = self.response_code {
            parts.push(format!("{} {}", self.version.as_ref().unwrap_or(&String::new()), code));
        }
        
        if let Some(host) = self.headers.get("Host") {
            parts.push(format!("Host: {}", host));
        }
        
        if let Some(ref body) = self.body {
            if body.len() > 100 {
                parts.push(format!("Body: {}...", &body[..100]));
            } else {
                parts.push(format!("Body: {}", body));
            }
        }
        
        parts.join(" | ")
    }
}

pub fn parse_tcp_packet(data: &[u8], src_port: u16, dst_port: u16) -> PacketInfo {
    let mut info = PacketInfo {
        src_port,
        dst_port,
        protocol: "TCP".to_string(),
        length: data.len(),
        ..Default::default()
    };
    
    if src_port == 80 || dst_port == 80 || src_port == 8080 || dst_port == 8080 {
        if let Some(http_info) = HttpInfo::parse_request(data) {
            info.protocol = "HTTP".to_string();
            info.http_info = Some(http_info);
        }
    } else if src_port == 443 || dst_port == 443 {
        info.protocol = "HTTPS".to_string();
    }
    
    info
}

pub fn parse_udp_packet(data: &[u8], src_port: u16, dst_port: u16) -> PacketInfo {
    let mut info = PacketInfo {
        src_port,
        dst_port,
        protocol: "UDP".to_string(),
        length: data.len(),
        ..Default::default()
    };
    
    if src_port == 53 || dst_port == 53 {
        info.protocol = "DNS".to_string();
    }
    
    info
}

pub fn format_packet_info(info: &PacketInfo) -> String {
    let mut lines = Vec::new();
    
    lines.push(format!(
        "[{}] {}:{} -> {}:{} ({} bytes)",
        info.timestamp,
        info.src_ip,
        info.src_port,
        info.dst_ip,
        info.dst_port,
        info.length
    ));
    
    lines.push(format!("  Protocol: {}", info.protocol));
    
    if let Some(ref http) = info.http_info {
        let summary = http.summary();
        if !summary.is_empty() {
            lines.push(format!("  {}", summary));
        }
    }
    
    lines.join("\n")
}