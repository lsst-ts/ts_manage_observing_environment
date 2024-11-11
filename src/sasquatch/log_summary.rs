use crate::error::ObsEnvError;
use chrono::Utc;
use std::{collections::BTreeMap, env};

pub trait AvroSchema {
    fn get_avro_schema(&self) -> String;
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Payload<T>
where
    T: AvroSchema,
{
    value_schema: String,
    records: Vec<Record<T>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Record<T> {
    value: T,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ActionData {
    timestamp: i64,
    action: String,
    repository: String,
    branch_name: String,
    user: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Summary {
    timestamp: i64,
    spectractor: String,
    atmospec: String,
    cwfs: String,
    summit_extras: String,
    summit_utils: String,
    ts_config_attcs: String,
    ts_config_ocs: String,
    ts_externalscripts: String,
    ts_observatory_control: String,
    ts_observing_utilities: String,
    ts_standardscripts: String,
    ts_wep: String,
}

impl AvroSchema for Summary {
    fn get_avro_schema(&self) -> String {
        r#"{"namespace": "lsst.obsenv","type": "record","name": "summary","fields": [{"name": "timestamp", "type": "long"},{"name": "spectractor", "type": "string"},{"name": "atmospec", "type": "string"},{"name": "cwfs", "type": "string"},{"name": "summit_extras", "type": "string"},{"name": "summit_utils", "type": "string"},{"name": "ts_config_attcs", "type": "string"},{"name": "ts_config_ocs", "type": "string"},{"name": "ts_externalscripts", "type": "string"},{"name": "ts_observatory_control", "type": "string"},{"name": "ts_observing_utilities", "type": "string"},{"name": "ts_standardscripts", "type": "string"},{"name": "ts_wep", "type": "string"}]}"#
        .to_owned()
    }
}

impl AvroSchema for ActionData {
    fn get_avro_schema(&self) -> String {
        r#"{"namespace": "lsst.obsenv","type": "record","name": "action","fields": [{"name": "timestamp", "type": "long"},{"name": "action", "type": "string"},{"name": "repository", "type": "string"},{"name": "branch_name", "type": "string"},{"name": "user", "type": "string"}]}"#.to_owned()
    }
}

macro_rules! extract_value {
    ($item:expr, $container:expr) => {
        if let Some(value) = $container.get($item) {
            match value {
                Ok(value) => value.to_owned(),
                Err(error) => error.to_string(),
            }
        } else {
            "Unknown".to_owned()
        }
    };
}

impl Summary {
    pub fn from_btree_map(summary: &BTreeMap<String, Result<String, ObsEnvError>>) -> Summary {
        let timestamp = Utc::now().timestamp_millis();
        let spectractor = extract_value!("spectractor", summary);
        let atmospec = extract_value!("atmospec", summary);
        let cwfs = extract_value!("cwfs", summary);
        let summit_extras = extract_value!("summit_extras", summary);
        let summit_utils = extract_value!("summit_utils ", summary);
        let ts_config_attcs = extract_value!("ts_config_attcs", summary);
        let ts_config_ocs = extract_value!("ts_config_ocs", summary);
        let ts_externalscripts = extract_value!("ts_externalscripts", summary);
        let ts_observatory_control = extract_value!("ts_observatory_control", summary);
        let ts_observing_utilities = extract_value!("ts_observing_utilities", summary);
        let ts_standardscripts = extract_value!("ts_standardscripts", summary);
        let ts_wep = extract_value!("ts_wep", summary);

        Summary {
            timestamp,
            spectractor,
            atmospec,
            cwfs,
            summit_extras,
            summit_utils,
            ts_config_attcs,
            ts_config_ocs,
            ts_externalscripts,
            ts_observatory_control,
            ts_observing_utilities,
            ts_standardscripts,
            ts_wep,
        }
    }
    pub fn get_topic_name() -> &'static str {
        "summary"
    }
}

impl ActionData {
    pub fn new(action: &str, repository: &str, branch_name: &str) -> ActionData {
        let user = match env::var("SUDO_USER") {
            Ok(val) => val,
            Err(_) => match env::var("USER") {
                Ok(val) => val,
                Err(_) => "Unknown".to_owned(),
            },
        };
        ActionData {
            timestamp: Utc::now().timestamp_millis(),
            action: action.to_owned(),
            repository: repository.to_owned(),
            branch_name: branch_name.to_owned(),
            user,
        }
    }
    pub fn get_topic_name() -> &'static str {
        "action"
    }
}

pub fn get_payload<T: AvroSchema>(record: T) -> Payload<T> {
    Payload {
        value_schema: record.get_avro_schema(),
        records: vec![Record { value: record }],
    }
}
