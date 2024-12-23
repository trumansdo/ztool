use log::debug;
use log4rs;
use serde_yml;

pub fn init_log4rs() {
    let config_str = include_str!("../config/log4rs.yaml");
    let config = serde_yml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();

    debug!("log4rs initialized");
}
