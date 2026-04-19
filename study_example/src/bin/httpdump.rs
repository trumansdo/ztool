use std::error::Error;

use study_example::netdump::httpdump;

fn main() -> Result<(), Box<dyn Error>> {
    let _ = httpdump::run();
    Ok(())
}
