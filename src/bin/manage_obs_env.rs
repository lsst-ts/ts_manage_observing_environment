use clap::Parser;
use simple_logger::SimpleLogger;
use std::process;
use ts_observing_environment::manage_obs_env::{run, ManageObsEnv};

fn main() {
    SimpleLogger::new().init().unwrap();

    let args = ManageObsEnv::parse();

    if let Err(e) = run(&args) {
        eprintln!("Application error: {:?}", e);
        process::exit(1);
    }
}
