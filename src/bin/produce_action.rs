use apache_avro::types::Value;
use rdkafka::{
    producer::{BaseProducer, BaseRecord},
    ClientConfig,
};
use schema_registry_converter::{
    blocking::{
        avro::AvroEncoder,
        schema_registry::{post_schema, SrSettings},
    },
    schema_registry_common::{SchemaType, SubjectNameStrategy, SuppliedSchema},
};
use simple_logger::SimpleLogger;
use std::time::Duration;
use ts_observing_environment::{
    obs_env_sidecar::{get_client_hosts, get_schema_registry_url},
    sasquatch::log_summary::{ActionData, AvroSchema},
};

/// Produce action.
///
/// This is a test cli to write the action topic directly to kafka.
///
fn main() {
    SimpleLogger::new().init().unwrap();

    let sr_settings = SrSettings::new(get_schema_registry_url());
    let subject_name = "lsst.obsenv.action-value";
    let topic_name = "lsst.obsenv.action";
    let action_data = ActionData::default();
    let supplied_schema = SuppliedSchema {
        name: Some(subject_name.to_string()),
        schema_type: SchemaType::Avro,
        schema: action_data.get_avro_schema(),
        references: vec![],
    };

    let _ = post_schema(&sr_settings, subject_name.to_string(), supplied_schema);

    let avro_encoder = AvroEncoder::new(sr_settings);

    let key_strategy =
        SubjectNameStrategy::TopicRecordNameStrategy(topic_name.to_string(), "value".to_string());
    // let mut kafka_producer = Producer::from_hosts(vec![get_client_hosts()])
    //     .create()
    let kafka_producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", get_client_hosts())
        .create()
        .expect("Could not create kafka producer.");
    let data_fields: Vec<(&str, Value)> = vec![
        ("timestamp", Value::Int(123)),
        ("action", Value::String("checkout-branch".to_string())),
        ("repository", Value::String("ts_config_ocs".to_string())),
        ("branch_name", Value::String("develop".to_string())),
        ("user", Value::String("tribeiro".to_string())),
    ];

    match avro_encoder.encode(data_fields, &key_strategy) {
        Ok(bytes) => {
            match kafka_producer.send(
                BaseRecord::to(topic_name)
                    .key("{{ \"name\": \"lsst.obsenv.action\" }}")
                    .payload(&bytes),
            ) {
                Ok(_) => log::info!("Ok"),
                Err((error, _)) => log::error!("Failed to send data: {error}"),
            }
        }
        Err(error) => log::error!("Failed to encode data: {error}"),
    }
    kafka_producer.poll(Duration::from_secs(1));
}
