//! # netstat — 全协议网络状态监控工具
//!
//! 比 tcpdump 功能更全面的网络抓包工具：支持 ARP/ICMP/ICMPv6 协议解析，
//! 处理 macOS/iOS 平台特殊的 TUN/loopback 接口，按协议类型分发到对应的 handle_*_packet 函数。
//!
//! ## Rust 概念 — `cfg!()` 条件编译宏
//! `cfg!(any(target_os = "macos", target_os = "ios"))` 是运行时条件判断宏，
//! 用于处理 macOS/iOS 平台特殊的 TUN 接口（没有原生以太网帧头，需手动构造）。
//!
//! ## Rust 概念 — `#[cfg(test)]` 条件编译
//! 测试模块只在运行 `cargo test` 时编译，不影响发布版本体积。
//!
//! ## Rust 概念 — `IpAddr` 枚举
//! `std::net::IpAddr` 统一表示 IPv4 和 IPv6 地址：
//! - `IpAddr::V4(addr)` — IPv4 变体
//! - `IpAddr::V6(addr)` — IPv6 变体
//! 使用 match 模式匹配来区分。
//!
//! ## Rust 概念 — `MutableEthernetPacket`
//! 可变数据包对象，允许修改字段后通过 `to_immutable()` 转为不可变引用。
//! 用于在内存中构造假的以太网帧头（TUN 接口不提供）。

use log::info;

use pnet::datalink::{self, Channel, NetworkInterface};

use pnet::packet::arp::ArpPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::icmp::{echo_reply, echo_request, IcmpPacket, IcmpTypes};
use pnet::packet::icmpv6::Icmpv6Packet;
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use pnet::util::MacAddr;

use std::net::IpAddr;

/// 处理 UDP 数据包
///
/// ## Rust 概念 — `as_ref()`
/// `udp.as_ref().unwrap().payload()` — as_ref() 将 Option<T> 转为 Option<&T>，
/// 链式调用 unwrap() 然后 payload()。用于在 if let 之前预先打印 payload 内容。
///
/// ## Rust 概念 — `String::from_utf8()`
/// 将 Vec<u8> 尝试转为 String。如果包含非法 UTF-8 字节则返回 Err。
/// `.to_vec()` — 将字节切片 &[u8] 转为 Vec<u8>（堆分配）。
fn handle_udp_packet(interface_name: &str, source: IpAddr, destination: IpAddr, packet: &[u8]) {
    let udp = UdpPacket::new(packet);
    println!(
        "UdpPacket Content: {:?}",
        String::from_utf8(
            udp.as_ref()
                .unwrap()
                .payload()
                .to_vec()
        )
    );
    if let Some(udp) = udp {
        println!(
            "[{}]: UDP Packet: {}:{} > {}:{}; length: {}",
            interface_name,
            source,
            udp.get_source(),
            destination,
            udp.get_destination(),
            udp.get_length()
        );
    } else {
        println!("[{}]: Malformed UDP Packet", interface_name);
    }
}

/// 处理 ICMP 数据包（ping 使用的协议）
///
/// ## Rust 概念 — 子类型解构
/// ICMP 有多种子类型（Echo Reply / Echo Request / ...），
/// 通过 `match icmp_packet.get_icmp_type()` 匹配并解构对应的子包结构。
///
/// ## Rust 概念 — `echo_reply::EchoReplyPacket::new()`
/// 从同一原始数据构造子类型的专用包对象，访问子类型特有的字段
/// 如 `get_sequence_number()` 和 `get_identifier()`。
fn handle_icmp_packet(interface_name: &str, source: IpAddr, destination: IpAddr, packet: &[u8]) {
    let icmp_packet = IcmpPacket::new(packet);
    if let Some(icmp_packet) = icmp_packet {
        match icmp_packet.get_icmp_type() {
            IcmpTypes::EchoReply => {
                let echo_reply_packet = echo_reply::EchoReplyPacket::new(packet).unwrap();
                println!(
                    "[{}]: ICMP echo reply {} -> {} (seq={:?}, id={:?})",
                    interface_name,
                    source,
                    destination,
                    echo_reply_packet.get_sequence_number(),
                    echo_reply_packet.get_identifier()
                );
            }
            IcmpTypes::EchoRequest => {
                let echo_request_packet = echo_request::EchoRequestPacket::new(packet).unwrap();
                println!(
                    "[{}]: ICMP echo request {} -> {} (seq={:?}, id={:?})",
                    interface_name,
                    source,
                    destination,
                    echo_request_packet.get_sequence_number(),
                    echo_request_packet.get_identifier()
                );
            }
            _ => println!(
                "[{}]: ICMP packet {} -> {} (type={:?})",
                interface_name,
                source,
                destination,
                icmp_packet.get_icmp_type()
            ),
        }
    } else {
        println!("[{}]: Malformed ICMP Packet", interface_name);
    }
}

/// 处理 ICMPv6 数据包（IPv6 的控制协议）
fn handle_icmpv6_packet(interface_name: &str, source: IpAddr, destination: IpAddr, packet: &[u8]) {
    let icmpv6_packet = Icmpv6Packet::new(packet);
    if let Some(icmpv6_packet) = icmpv6_packet {
        println!(
            "[{}]: ICMPv6 packet {} -> {} (type={:?})",
            interface_name,
            source,
            destination,
            icmpv6_packet.get_icmpv6_type()
        )
    } else {
        println!("[{}]: Malformed ICMPv6 Packet", interface_name);
    }
}

/// 处理 TCP 数据包
///
/// ## Rust 概念 — `println!` 的 `{:?}` 格式化
/// `println!("TcpPacket Content: {:?}", tcp);` — {:?} 是 Debug 格式，
/// 要求类型实现了 Debug trait。pnet 的 TcpPacket 实现了 Debug。
fn handle_tcp_packet(interface_name: &str, source: IpAddr, destination: IpAddr, packet: &[u8]) {
    let tcp = TcpPacket::new(packet);
    println!("TcpPacket Content: {:?}", tcp);
    if let Some(tcp) = tcp {
        println!(
            "[{}]: TCP Packet: {}:{} > {}:{}; length: {}",
            interface_name,
            source,
            tcp.get_source(),
            destination,
            tcp.get_destination(),
            packet.len()
        );
    } else {
        println!("[{}]: Malformed TCP Packet", interface_name);
    }
}

/// 传输层协议分发器
///
/// ## Rust 概念 — `match` 协议分发
/// 通过 `protocol` 参数匹配到对应的 handler 函数。
/// 这等效于 C 中的函数指针表/虚函数，但 Rust 用 match 实现更安全
/// （编译器会检查穷尽性）。
///
/// ## Rust 概念 — 内联 match 表达式
/// 在 format! 宏中嵌入 match 表达式：
/// ```
/// match source { IpAddr::V4(..) => "IPv4", _ => "IPv6" }
/// ```
/// match 是表达式，可以直接作为 format! 的参数。
fn handle_transport_protocol(
    interface_name: &str,
    source: IpAddr,
    destination: IpAddr,
    protocol: IpNextHeaderProtocol,
    packet: &[u8],
) {
    match protocol {
        IpNextHeaderProtocols::Udp => handle_udp_packet(interface_name, source, destination, packet),
        IpNextHeaderProtocols::Tcp => handle_tcp_packet(interface_name, source, destination, packet),
        IpNextHeaderProtocols::Icmp => handle_icmp_packet(interface_name, source, destination, packet),
        IpNextHeaderProtocols::Icmpv6 => handle_icmpv6_packet(interface_name, source, destination, packet),
        _ => println!(
            "[{}]: Unknown {} packet: {} > {}; protocol: {:?} length: {}",
            interface_name,
            match source {
                IpAddr::V4(..) => "IPv4",
                _ => "IPv6",
            },
            source,
            destination,
            protocol,
            packet.len()
        ),
    }
}

fn handle_ipv4_packet(interface_name: &str, ethernet: &EthernetPacket) {
    let header = Ipv4Packet::new(ethernet.payload());
    if let Some(header) = header {
        // IpAddr::V4() — 构造 IPv4 变体
        handle_transport_protocol(
            interface_name,
            IpAddr::V4(header.get_source()),
            IpAddr::V4(header.get_destination()),
            header.get_next_level_protocol(),
            header.payload(),
        );
    } else {
        println!("[{}]: Malformed IPv4 Packet", interface_name);
    }
}

fn handle_ipv6_packet(interface_name: &str, ethernet: &EthernetPacket) {
    let header = Ipv6Packet::new(ethernet.payload());
    if let Some(header) = header {
        handle_transport_protocol(
            interface_name,
            IpAddr::V6(header.get_source()),
            IpAddr::V6(header.get_destination()),
            header.get_next_header(),
            header.payload(),
        );
    } else {
        println!("[{}]: Malformed IPv6 Packet", interface_name);
    }
}

/// 处理 ARP 数据包（地址解析协议）
///
/// ARP 用于在局域网中将 IP 地址映射到 MAC 地址。
/// 例如设备想知道 "192.168.1.1 的 MAC 地址是什么" 时发送 ARP 请求。
fn handle_arp_packet(interface_name: &str, ethernet: &EthernetPacket) {
    let header = ArpPacket::new(ethernet.payload());
    if let Some(header) = header {
        println!(
            "[{}]: ARP packet: {}({}) > {}({}); operation: {:?}",
            interface_name,
            ethernet.get_source(),           // MAC 源地址
            header.get_sender_proto_addr(),  // IP 源地址（ARP 层）
            ethernet.get_destination(),       // MAC 目标地址
            header.get_target_proto_addr(),   // IP 目标地址（ARP 层）
            header.get_operation()            // 操作类型（请求/应答）
        );
    } else {
        println!("[{}]: Malformed ARP Packet", interface_name);
    }
}

/// 以太网帧处理分发器
///
/// 根据以太网帧的 EtherType 字段分发到对应的处理函数：
/// - Ipv4 → handle_ipv4_packet
/// - Ipv6 → handle_ipv6_packet
/// - Arp → handle_arp_packet
fn handle_ethernet_frame(interface: &NetworkInterface, ethernet: &EthernetPacket) {
    // &interface.name[..] — 将 String 转为 &str 切片
    let interface_name = &interface.name[..];
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(interface_name, ethernet),
        EtherTypes::Ipv6 => handle_ipv6_packet(interface_name, ethernet),
        EtherTypes::Arp => handle_arp_packet(interface_name, ethernet),
        _ => println!(
            "[{}]: Unknown packet: {} > {}; ethertype: {:?} length: {}",
            interface_name,
            ethernet.get_source(),
            ethernet.get_destination(),
            ethernet.get_ethertype(),
            ethernet
                .packet()
                .len()
        ),
    }
}

fn main() {
    info!("Starting packetdump");

    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .nth(0)
        .unwrap_or_else(|| panic!("No such network interface:"));

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("packetdump: unhandled channel type"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };

    // 主循环 — 持续接收数据包
    loop {
        // 预分配 1600 字节缓冲区（典型的 MTU 大小）
        let mut buf: [u8; 1600] = [0u8; 1600];  // [初始值; 长度] 数组重复语法
        let mut fake_ethernet_frame = MutableEthernetPacket::new(&mut buf[..]).unwrap();

        match rx.next() {
            Ok(packet) => {
                let payload_offset;

                // cfg!() 运行时平台检测宏
                // macOS/iOS 的 TUN/loopback 接口不提供原生以太网帧头
                // 需要手动构造假的以太网帧头
                if cfg!(any(target_os = "macos", target_os = "ios"))
                    && interface.is_up()           // 接口已启动
                    && !interface.is_broadcast()   // 非广播接口
                    && ((!interface.is_loopback() && interface.is_point_to_point()) || interface.is_loopback())
                {
                    if interface.is_loopback() {
                        // BPF loopback 添加了一个全零的以太网帧头（14 字节）
                        payload_offset = 14;
                    } else {
                        // 可能是 TUN 接口（无以太网帧头）
                        payload_offset = 0;
                    }
                    if packet.len() > payload_offset {
                        // 检查 IP 版本以设置正确的 EtherType
                        let version = Ipv4Packet::new(&packet[payload_offset..])
                            .unwrap()
                            .get_version();
                        if version == 4 {
                            // 手动构造假的以太网帧头
                            fake_ethernet_frame.set_destination(MacAddr(0, 0, 0, 0, 0, 0));
                            fake_ethernet_frame.set_source(MacAddr(0, 0, 0, 0, 0, 0));
                            fake_ethernet_frame.set_ethertype(EtherTypes::Ipv4);
                            fake_ethernet_frame.set_payload(&packet[payload_offset..]);
                            // to_immutable() — 可变 → 不可变引用转换
                            handle_ethernet_frame(&interface, &fake_ethernet_frame.to_immutable());
                            continue;  // 跳过正常处理流程
                        } else if version == 6 {
                            fake_ethernet_frame.set_destination(MacAddr(0, 0, 0, 0, 0, 0));
                            fake_ethernet_frame.set_source(MacAddr(0, 0, 0, 0, 0, 0));
                            fake_ethernet_frame.set_ethertype(EtherTypes::Ipv6);
                            fake_ethernet_frame.set_payload(&packet[payload_offset..]);
                            handle_ethernet_frame(&interface, &fake_ethernet_frame.to_immutable());
                            continue;
                        }
                    }
                }
                // 正常平台：直接用原始以太网帧
                handle_ethernet_frame(&interface, &EthernetPacket::new(packet).unwrap());
            }
            Err(e) => panic!("packetdump: unable to receive packet: {}", e),
        }
    }
}

// 测试模块 — 仅在 cargo test 时编译
#[cfg(test)]
mod tests {
    use pnet::datalink;
    #[test]
    fn test_interface_names() {
        // 枚举并打印所有网络接口
        datalink::interfaces()
            .into_iter()
            .for_each(|i| println!(" {:?}", i));
    }
}
