use clap::Command;
use clap::Parser;
use log::info;
use std::{error::Error, path::PathBuf};

use ztool::init::init_log4rs;

#[derive(Parser, Debug)]
#[command(name = "rgrep")]
#[command(version = "1.0")]
#[command(about = "rust mini grep tool", long_about = None)]
struct Cli {
    /// Sets a file
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    file: Option<PathBuf>,

    /// Sets a directory
    #[arg(short = 'd', long = "dir", value_name = "DIR")]
    dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    init_log4rs::init_log4rs();
    let cli = Cli::parse();
    info!("cli:{:?}", cli);
    Ok(())
}
