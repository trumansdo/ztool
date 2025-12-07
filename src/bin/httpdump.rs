use std::error::Error;

use ztool::netdump::httpdump;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = httpdump::run();
    Ok(())
}