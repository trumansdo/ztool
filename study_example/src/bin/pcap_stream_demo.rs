use futures::StreamExt;
use pcap::{Active, Capture, Device, Error, PacketCodec, PacketStream};
use std::error;

pub struct SimpleDumpCodec;

#[derive(Debug, PartialEq, Eq)]
pub struct PacketOwned {
    pub header: pcap::PacketHeader,
    pub data: Box<[u8]>,
}

impl PacketCodec for SimpleDumpCodec {
    type Item = PacketOwned;

    fn decode(&mut self, pkt: pcap::Packet) -> Self::Item {
        println!("SimplePacketCodec decode");
        PacketOwned {
            header: *pkt.header,
            data: pkt.data.into(),
        }
    }
}

async fn start_new_stream(device: Device) -> PacketStream<Active, SimpleDumpCodec> {
    match new_stream(device) {
        Ok(stream) => stream,
        Err(e) => {
            println!("{:?}", e);
            std::process::exit(1);
        }
    }
}

fn new_stream(device: Device) -> Result<PacketStream<Active, SimpleDumpCodec>, Error> {
    // get the default Device
    println!("Using device {}", device.name);

    let mut cap = Capture::from_device(device)?
        .immediate_mode(false)
        .promisc(true)
        .open()?
        .setnonblock()?;
    let _ = cap.filter("ip host 211.149.224.47 and tcp", true);
    cap.stream(SimpleDumpCodec {})
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let inter_name = "F0F07CFD-D6FE-49C9-8993-8790AFB777A2";

    let device = Device::list()?
        .into_iter()
        .find(|d| d.name.contains(inter_name))
        .unwrap();
    let stream = start_new_stream(device).await;

    let fut = stream.for_each(move |s| {
        println!("{:?}", s);
        futures::future::ready(())
    });
    fut.await;
    Ok(())
}