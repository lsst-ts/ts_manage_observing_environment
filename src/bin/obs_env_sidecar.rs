use clap::Parser;
use simple_logger::SimpleLogger;
use std::process;
use ts_observing_environment::obs_env_sidecar::{run, ObsEnvSidecar};

/// Observing Environment Sidecar application.
///
/// The sidecar applicationis designed to run as a daemon.
/// When started, it will setup the observing environment,
/// same as running manage_obs_env --action setup, then
/// it will monitor the actions logged to sasquatch and will
/// replicated them locally.
///
fn main() {
    SimpleLogger::new().init().unwrap();

    let obs_env_sidecar = ObsEnvSidecar::parse();

    if let Err(e) = run(&obs_env_sidecar) {
        eprintln!("Application error: {:?}", e);
        process::exit(1);
    }
}
