use crate::observing_environment::ObservingEnvironment;
use clap::{Parser, ValueEnum};
use log;
use std::error::Error;

/// Manage observing environment.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ManageObsEnv {
    /// Which action to execute?
    #[arg(value_enum, long = "action")]
    action: Action,
    /// Log level.
    #[arg(value_enum, long = "log-level")]
    log_level: LogLevel,
    /// Path to the environment.
    #[arg(long = "env-path", default_value = "/obs-env")]
    env_path: String,
}
pub trait ManageObsEnvCli {
    fn get_action(&self) -> &Action;
    fn get_log_level(&self) -> &LogLevel;
    fn get_env_path(&self) -> &str;
}

impl ManageObsEnvCli for ManageObsEnv {
    fn get_action(&self) -> &Action {
        &self.action
    }
    fn get_log_level(&self) -> &LogLevel {
        &self.log_level
    }
    fn get_env_path(&self) -> &str {
        &self.env_path
    }
}

pub fn run<T>(config: &T) -> Result<(), Box<dyn Error>>
where
    T: ManageObsEnvCli,
{
    match config.get_log_level() {
        LogLevel::Trace => log::set_max_level(log::LevelFilter::Trace),
        LogLevel::Debug => log::set_max_level(log::LevelFilter::Debug),
        LogLevel::Info => log::set_max_level(log::LevelFilter::Info),
        LogLevel::Warn => log::set_max_level(log::LevelFilter::Warn),
        LogLevel::Error => log::set_max_level(log::LevelFilter::Error),
    };

    log::info!("Running manage obs env...");

    let obs_env = ObservingEnvironment::with_destination(config.get_env_path());

    match config.get_action() {
        Action::Setup => {
            log::info!("Executing Setup...");

            log::debug!("Creating path...");
            obs_env.create_path()?;

            log::debug!("Cloning repositories...");
            let cloned_repos = obs_env.clone_repositories();
            log::info!("The following repositories where cloned: ");
            for repo in cloned_repos.iter() {
                match repo {
                    Ok(repo) => log::info!("{:?}", repo.path()),
                    Err(error) => log::error!("Failed to clone: {error:?}"),
                }
            }
        }
        Action::PrintConfig => {
            log::info!("{}", obs_env.summarize());
        }
        Action::Reset => {
            log::info!("Resetting Observing environment...");
            if let Err(error) = obs_env.reset_base_environment() {
                log::error!("Error resetting {} repositories.", error.len());
                for err in error {
                    log::error!("{:?}", err);
                }
            } else {
                log::info!("All repositories set to they base versions.");
            }
        }
        Action::ShowCurrentVersions => log::info!("Executing ShowCurrentVersions..."),
        Action::ShowOriginalVersions => log::info!("Executing ShowOriginalVersions..."),
    };
    Ok(())
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Action {
    /// Setup the observing environment?
    /// This will create the destination directory and clone all repositories.
    Setup,
    /// Show observing environment configuration?
    /// This will only print the observing environment configuration.
    PrintConfig,
    /// Reset obs environment. This will bring all repositories in the
    /// environment to their original versions.
    Reset,
    /// Show current versions.
    ShowCurrentVersions,
    /// Show original versions.
    ShowOriginalVersions,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
