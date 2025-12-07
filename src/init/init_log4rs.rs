use log::trace;
use log4rs;
use serde_yml;

pub fn init() {
    let config_str = include_str!("../config/log4rs.yaml");
    let config = serde_yml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();
    trace!("log4rs init success");
}
