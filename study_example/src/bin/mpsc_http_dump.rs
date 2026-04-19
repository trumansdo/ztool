use std::sync::mpsc;
use std::{error::Error, thread};

use pcap::{Capture, Device};

fn main() -> Result<(), Box<dyn Error>> {
    let _ = run();
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct PacketOwned {
    pub header: pcap::PacketHeader,
    pub data: Box<[u8]>,
}

fn run() -> Result<(), Box<dyn Error>> {
    // http://www.zhengbang.com/static/js/view/getWidth.js
    // http://www.zhengbang.com/static/images/prev.png  大概是1.8k  限制在1414byte一个数据包就可以分包了
    // curl -o  http://www.zhengbang.com/static/uploads/20220823/385e649fe80db74d9e46063f68b5b490.jpg
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
        let x: PacketOwned = recevier.recv().unwrap();
        println!("receive packet: {:?}", x.header);
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
