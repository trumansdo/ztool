use pnet::datalink;
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

use packets::GettableEndPoints;

const WIDTH: usize = 20;

fn main() {
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .nth(0)
        .unwrap_or_else(|| panic!("No such network interface:"));

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        // Ok(_) => panic!("Unhandled channel type"),
        Err(e) => {
            panic!("Failed to create datalink channel {}", e)
        }
        _ => panic!("Failed to create datalink channel"),
    };

    loop {
        match rx.next() {
            Ok(frame) => {
                let frame = EthernetPacket::new(frame).unwrap();
                match frame.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        ipv4_handler(&frame);
                    }
                    EtherTypes::Ipv6 => {
                        ipv6_handler(&frame);
                    }
                    _ => {
                        println!("Not a ipv4 or ipv6");
                    }
                }
            }
            Err(e) => {
                panic!("Failed to read: {}", e);
            }
        }
    }
}

fn print_packet_info(l3: &dyn GettableEndPoints, l4: &dyn GettableEndPoints, proto: &str) {
    println!(
        "Captured a {} packet from {}|{} to {}|{}\n",
        proto,
        l3.get_source(),
        l4.get_source(),
        l3.get_destination(),
        l4.get_destination()
    );
    let payload = l4.get_payload();
    let len = payload.len();

    for i in 0..len {
        print!("{:<02X} ", payload[i]);
        if i % WIDTH == WIDTH - 1 || i == len - 1 {
            for _j in 0..WIDTH - 1 - (i % (WIDTH)) {
                print!("   ");
            }
            print!("| ");
            for j in i - i % WIDTH..i + 1 {
                if payload[j].is_ascii_alphabetic() {
                    print!("{}", payload[j] as char);
                } else {
                    print!(".");
                }
            }
            print!("\n");
        }
    }
    println!("{}", "=".repeat(WIDTH * 3));
    print!("\n");
}

fn ipv4_handler(ethernet: &EthernetPacket) {
    if let Some(packet) = Ipv4Packet::new(ethernet.payload()) {
        match packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet);
            }
            IpNextHeaderProtocols::Udp => {
                udp_handler(&packet);
            }
            _ => {
                println!("Not a tcp or a udp packet");
            }
        }
    }
}

fn ipv6_handler(ethernet: &EthernetPacket) {
    if let Some(packet) = Ipv6Packet::new(ethernet.payload()) {
        match packet.get_next_header() {
            IpNextHeaderProtocols::Tcp => {
                tcp_handler(&packet);
            }
            IpNextHeaderProtocols::Udp => {
                udp_handler(&packet);
            }
            _ => {
                println!("Not a tcp or a udp packet");
            }
        }
    }
}

fn tcp_handler(packet: &dyn GettableEndPoints) {
    let tcp = TcpPacket::new(packet.get_payload());
    if let Some(tcp) = tcp {
        print_packet_info(packet, &tcp, "TCP");
    }
}

fn udp_handler(packet: &dyn GettableEndPoints) {
    let udp = UdpPacket::new(packet.get_payload());
    if let Some(udp) = udp {
        print_packet_info(packet, &udp, "UDP");
    }
}

mod packets {
    use pnet::packet::ipv4::Ipv4Packet;
    use pnet::packet::ipv6::Ipv6Packet;
    use pnet::packet::tcp::TcpPacket;
    use pnet::packet::udp::UdpPacket;
    use pnet::packet::Packet;

    pub trait GettableEndPoints {
        fn get_source(&self) -> String;
        fn get_destination(&self) -> String;
        fn get_payload(&self) -> &[u8];
    }

    impl<'a> GettableEndPoints for Ipv4Packet<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }

    impl<'a> GettableEndPoints for Ipv6Packet<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }

    impl<'a> GettableEndPoints for TcpPacket<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }

    impl<'a> GettableEndPoints for UdpPacket<'a> {
        fn get_source(&self) -> String {
            self.get_source()
                .to_string()
        }

        fn get_destination(&self) -> String {
            self.get_destination()
                .to_string()
        }

        fn get_payload(&self) -> &[u8] {
            self.payload()
        }
    }
}
