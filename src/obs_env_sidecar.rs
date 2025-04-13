use crate::{
    error::ObsEnvError,
    manage_obs_env::{run as run_manage_obs_env, LogLevel, ManageObsEnv},
    observing_environment::ObservingEnvironment,
    sasquatch::log_summary::ActionData,
};
use apache_avro::from_value;
use clap::Parser;
use log;
use rdkafka::{
    config::ClientConfig,
    consumer::{BaseConsumer, Consumer},
    Message,
};
use schema_registry_converter::blocking::{avro::AvroDecoder, schema_registry::SrSettings};
use std::{env, error::Error, process};

/// Implementation of the observing environment sidecar application.
///

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, name = "obs_env_sidecar")]
pub struct ObsEnvSidecar {
    /// Log level.
    #[arg(value_enum, long = "log-level", default_value = "debug")]
    log_level: LogLevel,
    /// Path to the environment.
    #[arg(long = "env-path", default_value = "/net/obs-env/auto_base_packages")]
    env_path: String,
}

impl ObsEnvSidecar {
    fn get_log_level(&self) -> &LogLevel {
        &self.log_level
    }

    fn get_env_path(&self) -> &str {
        &self.env_path
    }
}

pub fn run(config: &ObsEnvSidecar) -> Result<(), Box<dyn Error>> {
    if let Ok(_) = env::var("SASQUATCH_REST_PROXY_URL") {
        return Err(Box::new(ObsEnvError::ERROR("The SASQUATCH_REST_PROXY_URL environment variable cannot be set for the sidecar operation.".to_string())));
    }

    match config.get_log_level() {
        LogLevel::Trace => log::set_max_level(log::LevelFilter::Trace),
        LogLevel::Debug => log::set_max_level(log::LevelFilter::Debug),
        LogLevel::Info => log::set_max_level(log::LevelFilter::Info),
        LogLevel::Warn => log::set_max_level(log::LevelFilter::Warn),
        LogLevel::Error => log::set_max_level(log::LevelFilter::Error),
    };

    log::info!("Running obs_env_sidecar...");

    let obs_env = ObservingEnvironment::with_destination(config.get_env_path());

    log::info!("Setup obs_env...");

    log::debug!("Creating path...");
    obs_env.create_path()?;

    log::debug!("Cloning repositories...");
    let cloned_repos = obs_env.clone_repositories();

    log::debug!("The following repositories where cloned: ");
    for repo in cloned_repos.iter() {
        match repo {
            Ok(repo) => log::debug!("{:?}", repo.path()),
            Err(error) => log::error!("Failed to clone: {error:?}"),
        }
    }
    log::debug!("Creating setup file.");
    obs_env.create_setup_file()?;

    log::info!("Monitoring actions...");

    let client_config = {
        let mut client_config = ClientConfig::new();

        client_config
            .set("bootstrap.servers", get_client_hosts())
            .set("group.id", format!("example_group_{}", process::id()));

        if let (Ok(kafka_username), Ok(kafka_password)) = (
            env::var("OBS_ENV_KAFKA_SECURITY_USERNAME"),
            env::var("OBS_ENV_KAFKA_SECURITY_PASSWORD"),
        ) {
            log::info!("Using {kafka_username}::{kafka_password}");
            client_config
                .set(
                    "security.protocol",
                    env::var("LSST_KAFKA_SECURITY_PROTOCOL")
                        .unwrap_or("SASL_PLAINTEXT".to_string()),
                )
                .set(
                    "sasl.mechanism",
                    env::var("LSST_KAFKA_SECURITY_MECHANISM")
                        .unwrap_or("SCRAM-SHA-512".to_string()),
                )
                .set("sasl.username", kafka_username)
                .set("sasl.password", kafka_password);
        }
        client_config
    };

    let consumer: BaseConsumer = client_config.create()?;
    consumer
        .subscribe(&["lsst.obsenv.action"])
        .expect("Subscription failed");

    let sr_settings = SrSettings::new(get_schema_registry_url());
    let avro_decoder = AvroDecoder::new(sr_settings);

    loop {
        for message in consumer.iter() {
            match message {
                Ok(message) => {
                    let payload = message.payload();
                    let decoded_message = avro_decoder.decode(payload)?;
                    match from_value::<ActionData>(&decoded_message.value) {
                        Ok(action_data) => {
                            log::info!("Message {action_data:?}");
                            match action_data.get_action() {
                                Ok(action) => {
                                    let manage_obs_env = ManageObsEnv::default()
                                        .with_env_path(config.get_env_path())
                                        .with_action(action)
                                        .with_repository(&action_data.repository)
                                        .with_branch_name(&action_data.branch_name)
                                        .with_log_level(config.log_level.to_owned());
                                    log::info!("{manage_obs_env:?}");
                                    if let Err(e) = run_manage_obs_env(&manage_obs_env) {
                                        log::error!("Error running manage obs env: {e}.")
                                    }
                                }
                                Err(error) => log::error!("{error}"),
                            }
                        }
                        Err(error) => log::error!("Failed to decode message: {error}"),
                    }
                }
                Err(error) => log::info!("Error retrieving message: {error}"),
            }
        }
    }
}

pub fn get_client_hosts() -> String {
    if let Ok(client_host) = env::var("LSST_KAFKA_BROKER_ADDR") {
        client_host
    } else {
        "localhost:9092".to_string()
    }
}

pub fn get_schema_registry_url() -> String {
    if let Ok(url) = env::var("LSST_SCHEMA_REGISTRY_URL") {
        url
    } else {
        "http://127.0.0.1:8081".to_string()
    }
}
