use log::{error, info, trace};
use log4rs;
use serde_yml;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let config_str = include_str!("../config/log4rs.yaml");
    let config = serde_yml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();
    info!("Goes to console, file and rolling file");
    error!("Goes to console, file and rolling file");
    trace!("Doesn't go to console as it is filtered out");
    Ok(())
}
