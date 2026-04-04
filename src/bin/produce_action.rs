use apache_avro::types::Value;
use gethostname::gethostname;
use rand::{distr::Alphanumeric, Rng};
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic},
    client::DefaultClientContext,
    producer::{future_producer::FutureProducer, FutureRecord},
    ClientConfig,
};
use schema_registry_converter::{
    async_impl::{
        avro::AvroEncoder,
        schema_registry::{post_schema, SrSettings},
    },
    schema_registry_common::{SchemaType, SubjectNameStrategy, SuppliedSchema},
};
use simple_logger::SimpleLogger;
use std::{env, time::Duration};
use ts_observing_environment::{
    obs_env_sidecar::{get_client_hosts, get_schema_registry_url},
    sasquatch::log_summary::{ActionData, AvroSchema},
};

/// Produce action.
///
/// This is a test cli to write the action topic directly to kafka.
///
#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    log::info!("Producing action...");

    let sr_settings = SrSettings::new(get_schema_registry_url());
    let subject_name = "lsst.obsenv.action-value";
    let topic_name = "lsst.obsenv.action";
    let action_data = ActionData::default()
        .with_user("tribeiro")
        .with_action("checkout-branch")
        .with_repository("ts_config_ocs")
        .with_branch_name("develop")
        .with_timestamp(123);
    let supplied_schema = SuppliedSchema {
        name: Some(subject_name.to_string()),
        schema_type: SchemaType::Avro,
        schema: action_data.get_avro_schema(),
        references: vec![],
        properties: None,
        tags: None,
    };

    log::info!("Post schema");
    let _ = post_schema(&sr_settings, subject_name.to_string(), supplied_schema).await;

    let avro_encoder = AvroEncoder::new(sr_settings);

    let key_strategy =
        SubjectNameStrategy::TopicRecordNameStrategy(topic_name.to_string(), "value".to_string());
    // let mut kafka_producer = Producer::from_hosts(vec![get_client_hosts()])
    //     .create()

    let client_config = {
        let mut client_config = ClientConfig::new();
        let hostname: String = {
            if let Ok(hostname) = gethostname().into_string() {
                hostname
            } else {
                rand::rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect()
            }
        };

        client_config
            .set("bootstrap.servers", get_client_hosts())
            .set("group.id", format!("obs_env_sidecar_{}", hostname));

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

    let admin_client: AdminClient<DefaultClientContext> = client_config
        .create()
        .expect("Could not create admin client");

    let admin_options = AdminOptions::new();

    match admin_client
        .create_topics(
            &[NewTopic::new(
                topic_name,
                1,
                rdkafka::admin::TopicReplication::Fixed(1),
            )],
            &admin_options,
        )
        .await
    {
        Ok(create_topic_result) => {
            log::debug!("Created topic successfully {create_topic_result:?}")
        }
        Err(error) => log::error!("Error creating topic: {error}."),
    };

    let kafka_producer: FutureProducer = client_config
        .create()
        .expect("Could not create kafka producer.");

    match avro_encoder.encode_struct(action_data, &key_strategy).await {
        Ok(bytes) => {
            match kafka_producer
                .send(
                    FutureRecord::to(topic_name)
                        .key("{{ \"name\": \"lsst.obsenv.action\" }}")
                        .payload(&bytes),
                    Duration::from_secs(1),
                )
                .await
            {
                Ok(_) => log::info!("Ok"),
                Err((error, _)) => log::error!("Failed to send data: {error}"),
            }
        }
        Err(error) => log::error!("Failed to encode data: {error}"),
    }
    for _ in 0..10 {
        kafka_producer.poll(Duration::from_secs(1));
    }
}
