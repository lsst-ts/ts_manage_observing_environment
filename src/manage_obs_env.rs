use crate::{
    error::ObsEnvError,
    observing_environment::ObservingEnvironment,
    repos::Repos,
    sasquatch::{
        create_topic::create_topics,
        log_summary::{get_payload, ActionData, AvroSchema, Payload, Summary},
        run_branch::{self, RunBranch},
    },
};
use clap::Parser;
use log;
use reqwest;
use serde::ser::Serialize;
use std::{collections::BTreeMap, env, error::Error, fmt::Debug};

/// Manage observing environment.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, name = "manage_obs_env")]
pub struct ManageObsEnv {
    /// Which action to execute?
    #[arg(value_enum, long = "action")]
    action: Action,
    /// Log level.
    #[arg(value_enum, long = "log-level", default_value = "debug")]
    log_level: LogLevel,
    /// Path to the environment.
    #[arg(long = "env-path", default_value = "/net/obs-env/auto_base_packages")]
    env_path: String,
    /// Repository to act on (for actions on individual repos).
    #[arg(value_enum, long = "repository")]
    repository: Option<Repos>,
    /// Name of the branch or version to checkout when running the "CheckoutBranch"
    /// or "CheckoutVersion" action.
    #[arg(long = "branch-name", default_value = "")]
    branch_name: String,
    /// Name of the branch to checkout when running the "Reset"
    /// action.
    #[arg(long = "base-env-branch-name", default_value = "main")]
    base_env_branch_name: String,
}
pub trait ManageObsEnvCli {
    fn get_action(&self) -> Result<&Action, Box<dyn Error>>;
    fn get_log_level(&self) -> &LogLevel;
    fn get_env_path(&self) -> &str;
    fn get_branch_name(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_repository_name(&self) -> &str;
    fn get_base_env_source_repo(&self) -> &str;
}

impl ManageObsEnvCli for ManageObsEnv {
    fn get_action(&self) -> Result<&Action, Box<dyn Error>> {
        match self.action {
            Action::CheckoutBranch => {
                if self.repository.is_none() {
                    Err(Box::new(ObsEnvError::ERROR(
                        "Checkout branch action requires a repository, none given".to_owned(),
                    )))
                } else {
                    Ok(&self.action)
                }
            }
            _ => Ok(&self.action),
        }
    }
    fn get_log_level(&self) -> &LogLevel {
        &self.log_level
    }
    fn get_env_path(&self) -> &str {
        &self.env_path
    }
    fn get_branch_name(&self) -> &str {
        &self.branch_name
    }
    fn get_version(&self) -> &str {
        &self.branch_name
    }
    fn get_repository_name(&self) -> &str {
        if let Some(repository) = &self.repository {
            repository.get_name()
        } else {
            ""
        }
    }
    fn get_base_env_source_repo(&self) -> &str {
        &self.base_env_branch_name
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

    match config.get_action()? {
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
            log::info!("Creating setup file.");
            obs_env.create_setup_file()?;
            log::debug!("Sending action.");
            send_action_data("setup", "", "");
            log::debug!("Sending summary.");
            let current_versions = obs_env.get_current_env_versions();
            send_summary_data(&current_versions);
        }
        Action::PrintConfig => {
            log::info!("{}", obs_env.summarize());
        }
        Action::Reset => {
            log::info!("Resetting Observing environment...");
            let run_branch = {
                if let Ok(efd_name) = env::var("MANAGE_OBS_ENV_EFD_NAME") {
                    RunBranch::retrieve_from_efd(&efd_name)?
                        .get_branch_name()
                        .to_owned()
                } else {
                    "".to_owned()
                }
            };
            if let Err(error) =
                obs_env.reset_base_environment(config.get_base_env_source_repo(), &run_branch)
            {
                log::error!("Error resetting {} repositories.", error.len());
                for err in error {
                    log::error!("{:?}", err);
                }
            } else {
                log::info!("All repositories set to their base versions.");
            }
            log::debug!("Sending action.");
            send_action_data("reset", "", "");
            log::debug!("Sending summary.");
            let current_versions = obs_env.get_current_env_versions();
            send_summary_data(&current_versions);
        }
        Action::ShowCurrentVersions => {
            log::info!("Current environment versions:");
            let current_versions = obs_env.get_current_env_versions();
            for (name, version) in current_versions.iter() {
                match version {
                    Ok(version) => log::info!("{name}: {version}"),
                    Err(error) => log::error!("{name}: {error:?}"),
                }
            }
            log::debug!("Sending action.");
            send_action_data("show-current-versions", "", "");
        }
        Action::ShowOriginalVersions => {
            match obs_env.get_base_env_versions(config.get_base_env_source_repo()) {
                Ok(base_env_versions) => {
                    log::info!("Base Environment versions:");
                    for (name, version) in base_env_versions.iter() {
                        log::info!("{name}: {version}");
                    }
                }
                Err(error) => {
                    log::error!("{error:?}");
                }
            }
            log::debug!("Sending action.");
            send_action_data("show-original-versions", "", "");
        }
        Action::CheckoutBranch => {
            obs_env.checkout_branch(config.get_repository_name(), config.get_branch_name())?;
            log::debug!("Sending action.");
            send_action_data(
                "checkout-branch",
                config.get_repository_name(),
                config.get_branch_name(),
            );
            log::debug!("Sending summary.");
            let current_versions = obs_env.get_current_env_versions();
            send_summary_data(&current_versions);
        }
        Action::CheckoutVersion => {
            obs_env.reset_index_to_version(config.get_repository_name(), config.get_version())?;
            log::debug!("Sending action.");
            send_action_data(
                "checkout-version",
                config.get_repository_name(),
                config.get_version(),
            );
            log::debug!("Sending summary.");
            let current_versions = obs_env.get_current_env_versions();
            send_summary_data(&current_versions);
        }
        Action::CreateTopics => {
            if let Ok(sasquatch_rest_proxy_url) = env::var("SASQUATCH_REST_PROXY_URL") {
                create_topics(&sasquatch_rest_proxy_url)?
            } else {
                log::error!(
                    "Environment variable SASQUATCH_REST_PROXY_URL, not set. \
                    This variable defines the url of the sasquatch service and needs \
                    to be defined for the topics to be registered."
                );
            }
        }
        Action::RegisterRunBranch => {
            if let Ok(_) = env::var("SASQUATCH_REST_PROXY_URL") {
                log::info!("Registering run branch.");
                send_run_branch(&config.get_branch_name());
            } else {
                log::error!(
                    "In order to register the run branch you must setup SASQUATCH_REST_PROXY_URL."
                );
            }
            log::debug!("Sending action.");
            send_action_data("register-run-branch", "", &config.get_branch_name());
        }
        Action::ClearRunBranch => {
            if let Ok(_) = env::var("SASQUATCH_REST_PROXY_URL") {
                log::info!("Clearing run branch.");
                send_run_branch("");
            } else {
                log::error!(
                    "In order to clear the run branch you must setup SASQUATCH_REST_PROXY_URL."
                );
            }
            log::debug!("Sending action.");
            send_action_data("clear-run-branch", "", "");
        }
        Action::ListRunBranch => {
            if let Ok(efd_name) = env::var("MANAGE_OBS_ENV_EFD_NAME") {
                log::info!("Retrieving run branch from {efd_name} instance of the EFD.");
                let run_branch = RunBranch::retrieve_from_efd(&efd_name)?;
                log::info!("Current run branch: {}", run_branch.get_branch_name());
            } else {
                log::error!(
                    "In order to list the currently registered run branch you must setup the MANAGE_OBS_ENV_EFD_NAME environment variable with the name of the EFD instance for this environment."
                );
            }
            log::debug!("Sending action.");
            send_action_data("list-run-branch", "", "");
        }
        Action::CheckoutRunBranch => {
            if let Ok(efd_name) = env::var("MANAGE_OBS_ENV_EFD_NAME") {
                let run_branch = RunBranch::retrieve_from_efd(&efd_name)?;
                if run_branch.get_branch_name().len() > 0 {
                    log::info!(
                        "Checkout run branch ({}) for {}.",
                        run_branch.get_branch_name(),
                        config.get_repository_name()
                    );
                    obs_env.checkout_branch(
                        config.get_repository_name(),
                        run_branch.get_branch_name(),
                    )?;
                    log::debug!("Sending action.");
                    send_action_data(
                        "checkout-run-branch",
                        config.get_repository_name(),
                        run_branch.get_branch_name(),
                    );
                    log::debug!("Sending summary.");
                    let current_versions = obs_env.get_current_env_versions();
                    send_summary_data(&current_versions);
                } else {
                    log::error!("Currently no run branch registered.");
                }
            } else {
                log::error!(
                    "In order to checkout the currently registered run branch you must setup the MANAGE_OBS_ENV_EFD_NAME environment variable with the name of the EFD instance for this environment."
                );
            }
        }
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
    /// Checkout a branch in a repository.
    CheckoutBranch,
    /// Checkout a version in a repository.
    CheckoutVersion,
    /// Create topics to log data to sasquatch.
    CreateTopics,
    /// Register run branch.
    RegisterRunBranch,
    /// Clear the run branch.
    ClearRunBranch,
    /// List the currently registered run branch.
    ListRunBranch,
    /// Checkout the run branch for a specific repository.
    CheckoutRunBranch,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

fn send_summary_data(current_versions: &BTreeMap<String, Result<String, ObsEnvError>>) {
    let log_summary = Summary::from_btree_map(current_versions);
    let payload = get_payload(log_summary);
    send_payload(&payload, Summary::get_topic_name());
}

fn send_action_data(action: &str, repository: &str, branch_name: &str) {
    let action = ActionData::new(action, repository, branch_name);
    let payload = get_payload(action);
    send_payload(&payload, ActionData::get_topic_name());
}

fn send_run_branch(branch_name: &str) {
    let run_branch = RunBranch::new(branch_name);
    let payload = get_payload(run_branch);
    send_payload(&payload, RunBranch::get_topic_name());
}

fn send_payload<T: AvroSchema + Debug + Serialize>(payload: &Payload<T>, topic_name: &str) {
    let client = reqwest::blocking::Client::new();
    log::debug!("{topic_name}");
    if let Ok(sasquatch_rest_proxy_url) = env::var("SASQUATCH_REST_PROXY_URL") {
        if let Ok(res) = client
            .post(format!(
                "{sasquatch_rest_proxy_url}/sasquatch-rest-proxy/topics/lsst.obsenv.{topic_name}",
            ))
            .header("Content-Type", "application/vnd.kafka.avro.v2+json")
            .header("Accept", "application/vnd.kafka.v2+json")
            .json(payload)
            .send()
        {
            if !res.status().is_success() {
                log::error!("Server replied with error to payload request: {res:?}. {payload:?}");
            } else {
                log::trace!("Payload: {payload:?}.");
            }
        } else {
            log::error!("Error sending payload.");
        }
    } else {
        log::error!(
            "Environment variable SASQUATCH_REST_PROXY_URL, not set. \
            This variable defines the url of the sasquatch service and needs \
            to be defined for actions to be registered."
        )
    }
}
