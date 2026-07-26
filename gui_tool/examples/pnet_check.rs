//! pnet API 签名探查
//! 编译检查来确定确切的类型和方法签名

fn main() {
    // 检查 transport_channel 签名
    let _ = check_transport();
    // 检查 TcpPacket 和 MutableTcpPacket
    check_packet_api();
    // 检查 TcpFlags
    check_tcp_flags();
    println!("API check: OK");
}

use pnet::packet::tcp::{
    TcpPacket, MutableTcpPacket, ipv4_checksum,
    TcpFlags,
};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::transport::{
    transport_channel,
    TransportChannelType,
    TransportProtocol,
};
use std::net::Ipv4Addr;

fn check_transport() -> Result<(), Box<dyn std::error::Error>> {
    // 检查 TransportChannelType::Layer4 变体
    // 以及 TransportProtocol::Ipv4 需要什么参数
    let _protocol = TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp);
    let _chan_type = TransportChannelType::Layer4(_protocol);

    let (mut _sender, _receiver) = transport_channel(65536, _chan_type)?;

    // 检查 send_to 签名
    let data = vec![0u8; 20];
    let _ = _sender.send_to(&data, Ipv4Addr::new(192, 168, 1, 1));

    // 检查 recv_from 签名
    let mut buf = vec![0u8; 65536];
    let _ = _receiver.recv_from(&mut buf);

    Ok(())
}

fn check_packet_api() {
    // MutableTcpPacket::new 签名
    let mut buf = vec![0u8; 20];
    let mut pkt = MutableTcpPacket::new(&mut buf).unwrap();

    // 检查 setter 方法签名
    pkt.set_source(12345);
    pkt.set_destination(80);
    pkt.set_sequence(1000);
    pkt.set_acknowledgement(0);
    pkt.set_data_offset(5);
    pkt.set_flags(0x02); // SYN
    pkt.set_window(65535);
    pkt.set_checksum(0);
    pkt.set_urgent_ptr(0);

    // 检查 to_immutable
    let _imm = pkt.to_immutable();

    // 检查 TcpPacket::new 签名
    let _pkt2 = TcpPacket::new(&buf);

    // 检查 getter
    if let Some(imm) = TcpPacket::new(&buf) {
        let _src = imm.get_source();
        let _dst = imm.get_destination();
        let _seq = imm.get_sequence();
        let _flags = imm.get_flags();
    }

    // 检查 ipv4_checksum 签名
    if let Some(imm) = TcpPacket::new(&buf) {
        let _ = ipv4_checksum(
            &imm,
            &Ipv4Addr::new(192, 168, 1, 2),
            &Ipv4Addr::new(192, 168, 1, 1),
        );
    }
}

fn check_tcp_flags() {
    // 检查 TcpFlags 的值
    let _syn = TcpFlags::SYN;
    let _ack = TcpFlags::ACK;
    let _rst = TcpFlags::RST;
    let _fin = TcpFlags::FIN;
    let _psh = TcpFlags::PSH;
    let _urg = TcpFlags::URG;

    // 检查它们是否为 u8
    let _flags: u8 = TcpFlags::SYN | TcpFlags::ACK;
    println!("SYN={:02x}, ACK={:02x}, RST={:02x}", TcpFlags::SYN, TcpFlags::ACK, TcpFlags::RST);
}
