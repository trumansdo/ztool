use std::{error::Error, fmt::Display, io::Cursor, sync::mpsc, thread};

use byteorder::{BigEndian, ReadBytesExt};
use itertools::Itertools;
use pcap::{Capture, Device};
use pnet::packet::{
    ethernet::{EtherTypes, EthernetPacket},
    ip::IpNextHeaderProtocols,
    ipv4::{Ipv4Flags, Ipv4Packet},
    tcp::{TcpFlags, TcpOptionNumbers, TcpPacket}, Packet,
};

/**
* OSI七层模型
* -------> 应用层(HTTP, FTP, SMTP, DNS, Telnet, SNMP, NFS)
* ------> 表示层(SSL/TLS, MIME, ASCII, JPEG, GIF, MPEG)
* -----> 会话层(NFS, SQL, RPC, NetBIOS)
* ----> 传输层(TCP, UDP, SCTP)
* ---> 网络层(IP, ICMP, IGMP, ARP, RARP)
* --> 数据链路层(PPP, Ethernet, Frame Relay, HDLC)
* -> 物理层(Ethernet, Wi-Fi, DSL, ISDN, FDDI)

* TCP/IP四层模型
* ----> 应用层(HTTP, FTP, SMTP, DNS)
* ---> 传输层(TCP, UDP)
* --> 网络层(IP, ICMP, IGMP)
* -> 网络接口层(Ethernet, Wi-Fi, DSL, ATM)
*
* 在pnet中，有四层：
* Layer 2, datalink layer;  第 2 层，数据链路层； 默认此层
* Layer 3, network layer;  第 3 层，网络层；
* Layer 4, transport layer. 第 4 层，传输层。
*
* 在应用中，我们一般都是先通过以太网接口打开 数据链路层，
* 然后一层一层判断其header与payload，把payload交给对应的上层协议packet解析
* 详细报文格式可以参考华为提供的：
* https://support.huawei.com/enterprise/zh/doc/EDOC1100174722/fc60e39
*/
/// 定义一个结构体，用于保存各层的packet数据
#[derive(Debug, PartialEq)]
pub struct TcpHolisticPacket<'a> {
    identification: u16,
    frame_packet: &'a pcap::Packet<'a>,
    eth_packet: &'a EthernetPacket<'a>,
    ipv4_packet: &'a Ipv4Packet<'a>,
    tcp_packet: &'a TcpPacket<'a>,
}

pub trait PacketSummary {
    fn summary(&self) -> String;
}
impl<'a> PacketSummary for pcap::Packet<'a> {
    fn summary(&self) -> String {
        let sec = self.header.ts.tv_sec as u64;
        let usec = self.header.ts.tv_usec as u64;
        format!("Frame[Time:{} Len:{} CapLen:{}]", (sec * 1000000 + usec), self.header.len, self.header.caplen)
    }
}

impl<'a> PacketSummary for EthernetPacket<'a> {
    fn summary(&self) -> String {
        format!(
            "Ethernet[EtherType:{} Src:{}  Dst:{} PacketLen:{} PayloadLen:{}]",
            self.get_ethertype(),
            self.get_source(),
            self.get_destination(),
            self.packet().len(), //长度等于上一层的payload长度
            self.payload().len()
        )
    }
}

impl<'a> PacketSummary for Ipv4Packet<'a> {
    fn summary(&self) -> String {
        let ipv4_packet_flags = self.get_flags();

        format!(
            "NetWork[IPv4 Src:{} Dst:{} HeaderLen:{} Ident:{} Flags:[{} DF:{} MF:{}] FragOffset:{} TTL:{} \
 Proto:{} HeaderChecksum:{} Options:{:?} PacketLen:{} PayloadLen:{}]",
            self.get_source(),
            self.get_destination(),
            self.get_header_length(),
            self.get_identification(),
            0,
            ((Ipv4Flags::DontFragment & ipv4_packet_flags) >> 1),
            (Ipv4Flags::MoreFragments & ipv4_packet_flags),
            self.get_fragment_offset(),
            self.get_ttl(),
            self.get_next_level_protocol(),
            self.get_checksum(),
            self.get_options(),
            self.packet().len(), //长度等于上一层的payload长度
            self.payload().len()
        )
    }
}

impl<'a> PacketSummary for TcpPacket<'a> {
    fn summary(&self) -> String {
        let tcp_packet_flags = self.get_flags();
        format!(
            "Transport[TCP SrcPort:{} DstPort:{} Seq:{} Ack:{} DataOffset:{} Reserved:{} \
  Flags[CWR:{} ECE:{} URG:{} ACK:{} PSH:{} RST:{} SYN:{} FIN:{}] WindowSize:{} Checksum:{} \
  UrgentPointer:{} Options:[{}] PacketLen:{} PlayLoadLen:{} ",
            self.get_source(),
            self.get_destination(),
            self.get_sequence(),
            self.get_acknowledgement(),
            self.get_data_offset(),
            self.get_reserved(),
            ((TcpFlags::CWR & tcp_packet_flags) >> 7),
            ((TcpFlags::ECE & tcp_packet_flags) >> 6),
            ((TcpFlags::URG & tcp_packet_flags) >> 5),
            ((TcpFlags::ACK & tcp_packet_flags) >> 4),
            ((TcpFlags::PSH & tcp_packet_flags) >> 3),
            ((TcpFlags::RST & tcp_packet_flags) >> 2),
            ((TcpFlags::SYN & tcp_packet_flags) >> 1),
            (TcpFlags::FIN & tcp_packet_flags),
            self.get_window(),
            self.get_checksum(),
            self.get_urgent_ptr(),
            self.get_options()
                .into_iter()
                .map(|x| {
                    let mut val = 0;
                    if x.data.len() != 0 {
                        let mut cur = Cursor::new(x.data);
                        val = cur
                            .read_uint::<BigEndian>(cur.get_ref().len())
                            .unwrap();
                    }
                    match x.number {
                        TcpOptionNumbers::EOL => format!("EOL[len:{:?} val:{}]", x.length, val),
                        TcpOptionNumbers::NOP => format!("NOP[len:{:?} val:{}]", x.length, val),
                        TcpOptionNumbers::MSS => format!("MSS[len:{:?} val:{}]", x.length, val),
                        TcpOptionNumbers::WSCALE => format!("WSCALE[len:{:?} val:{}]", x.length, val),
                        TcpOptionNumbers::SACK_PERMITTED => format!("SACK_PERMITTED[len:{:?} val:{}]", x.length, val),
                        TcpOptionNumbers::SACK => format!("SACK[len:{:?} val:{}]", x.length, val),
                        TcpOptionNumbers::TIMESTAMPS => format!("TIMESTAMPS[len:{:?} val:{}]", x.length, val),
                        _ => "Unknown".to_string(),
                    }
                })
                .map(|x| format!("{} ", x))
                .collect::<String>(),
            self.packet().len(), //长度等于上一层的payload长度
            self.payload().len()
        )
    }
}

impl<'a> Display for TcpHolisticPacket<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "FrameLayer -> {}", self.frame_packet.summary());
        let _ = writeln!(f, "EthernetLayer -> {}", self.eth_packet.summary());
        let _ = writeln!(f, "NetWorkLayer -> {}", self.ipv4_packet.summary());
        let _ = writeln!(f, "TransportLayer -> {}", self.tcp_packet.summary());
        writeln!(f, "-----------------------------------------------------")
    }
}

fn handle_packet<'a>(frame_packet: PacketOwned) -> Result<(), Box<dyn Error>> {
    //物理层的frame数据包
    let fr_pck = pcap::Packet {data: &frame_packet.data,header: &frame_packet.header};

    // 链路层的数据包
    let eth_packet = EthernetPacket::new(fr_pck.data).unwrap();

    match eth_packet.get_ethertype() {
        // 网络层数据包，一般处理IPV4 和 IPV6
        EtherTypes::Ipv4 => {
            let ipv4_packet = Ipv4Packet::new(eth_packet.payload()).ok_or("Ipv4Packet::new failed")?;
            match ipv4_packet.get_next_level_protocol() {
                // 传输层数据包，一般处理TCP 和 UDP
                IpNextHeaderProtocols::Tcp => {
                    let tcp_packet = TcpPacket::new(ipv4_packet.payload()).ok_or("TcpPacket::new failed")?;
                    let tcp_payload = tcp_packet.payload();
                    let http_header = tcp_payload
                        .into_iter()
                        .tuple_windows::<(_, _, _, _)>()
                        .position(|x| x == (&13u8, &10u8, &13u8, &10u8))
                        .map(|i| &tcp_payload[0..i])
                        .map(|x| String::from_utf8(x.to_vec()))
                        .ok_or("no CRLF found");
                    println!(
                        "{}",
                        TcpHolisticPacket {
                            identification: ipv4_packet.get_identification(),
                            frame_packet: &fr_pck,
                            eth_packet: &eth_packet,
                            ipv4_packet: &ipv4_packet,
                            tcp_packet: &tcp_packet,
                        }
                    );
                }
                _ => {
                    // println!("======暂时不处理{}协议,{:?}", next_level_protocol,ipv4_packet);
                }
            }
        }
        _ => {
            // println!("===暂时不处理{}协议,{:?}", eth_packet.get_ethertype(),eth_packet);
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct PacketOwned {
    pub header: pcap::PacketHeader,
    pub data: Box<[u8]>,
}


pub fn run() -> Result<(), Box<dyn Error>> {
    // http://www.zhengbang.com/static/js/view/getWidth.js
    // http://www.zhengbang.com/static/images/prev.png  大概是1.8k  限制在1414byte一个数据包就可以分包了
    println!("start capture");
    let inter_name = "F0F07CFD-D6FE-49C9-8993-8790AFB777A2";

    let device = Device::list()?
        .into_iter()
        .find(|d| d.name.contains(inter_name))
        .unwrap();

    let mut cap = Capture::from_device(device)?
        .immediate_mode(true)
        .promisc(true)
        .open()?
        .setnonblock()?;
    cap.filter("ip host 211.149.224.47 and tcp", true)?;
    let (sender, recevier) = mpsc::channel();
    thread::spawn(move || loop {
        match recevier.recv() {
            Ok(packet) => {
                let _ = handle_packet(packet);
            }
            Err(_) => {}
        }
    });
    loop {
        match cap.next_packet() {
            Ok(packet) => {
                let _ = sender.send(PacketOwned {
                    header: *packet.header,
                    data: Box::from(packet.data),
                });
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use pnet::{datalink, packet::tcp::TcpOptionNumbers};

    #[test]
    fn it_works() {
        // Device { name: "\\Device\\NPF_{2CDA97A7-5E8F-4F7F-AD6B-0D06400EFF83}",  addresses: [Address { addr: 10.89.123.107, netmask: Some(255.255.255.128), broadcast_addr: Some(10.89.123.127), dst_addr: None }] }
        // { name: "\\Device\\NPF_{F0F07CFD-D6FE-49C9-8993-8790AFB777A2}", ips: [V4(Ipv4Network { addr: 192.168.100.104, prefix: 24 })], flags: 0 }
        datalink::interfaces()
            .into_iter()
            .for_each(|d| println!("{:?}", d));
    }

    #[test]
    fn tcp_options() {
        println!("{:?}", TcpOptionNumbers::EOL);
        println!("{:?}", TcpOptionNumbers::NOP);
        println!("{:?}", TcpOptionNumbers::MSS);
        println!("{:?}", TcpOptionNumbers::WSCALE);
        println!("{:?}", TcpOptionNumbers::SACK_PERMITTED);
        println!("{:?}", TcpOptionNumbers::SACK);
        println!("{:?}", TcpOptionNumbers::TIMESTAMPS);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 10)] // 使用多线程运行时，线程池大小为 4
    async fn tokio_test() {
        use std::time::Instant;
        use tokio::time::{sleep, Duration};

        let start = Instant::now();
        println!("main thread id: {:?}", std::thread::current().id());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());
        tokio::spawn(async_test());

        println!("All tasks completed in {:?} {:?}", start.elapsed(), std::thread::current().id());
    }
    async fn async_test() {
        
        use std::time::Instant;
        use tokio::time::{sleep, Duration};        
        println!("async_test started on thread {:?} at time {:?}", std::thread::current().id(), Instant::now());
        sleep(Duration::from_secs(1)).await;
        println!("async_test finished on thread {:?} at time {:?}", std::thread::current().id(), Instant::now());
    }
}
