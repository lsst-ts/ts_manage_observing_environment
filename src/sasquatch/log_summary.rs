use crate::{error::ObsEnvError, manage_obs_env::Action};
use chrono::Utc;
use std::{collections::BTreeMap, env};
use thiserror::Error as ThisError;

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
    pub repository: String,
    pub branch_name: String,
    user: String,
}

#[derive(Clone, Debug, Eq, ThisError, PartialEq)]
#[error("{0}")]
pub struct ErrorGettingAction(String);

impl ActionData {
    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }
    pub fn with_action(mut self, action: &str) -> Self {
        self.action = action.to_owned();
        self
    }
    pub fn with_repository(mut self, repository: &str) -> Self {
        self.repository = repository.to_owned();
        self
    }
    pub fn with_branch_name(mut self, branch_name: &str) -> Self {
        self.branch_name = branch_name.to_owned();
        self
    }
    pub fn with_user(mut self, user: &str) -> Self {
        self.user = user.to_owned();
        self
    }
    pub fn get_action(&self) -> Result<Action, ErrorGettingAction> {
        if self.action == "checkout-branch" || self.action == "checkout-run-branch" {
            Ok(Action::CheckoutBranch)
        } else if self.action == "reset" {
            Ok(Action::Reset)
        } else {
            Err(ErrorGettingAction(format!(
                "Unsupported action: {}",
                self.action
            )))
        }
    }
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
    ts_config_mttcs: String,
    ts_config_ocs: String,
    ts_config_scheduler: String,
    ts_externalscripts: String,
    ts_observatory_control: String,
    ts_observing_utilities: String,
    ts_standardscripts: String,
    ts_maintel_standardscripts: String,
    ts_auxtel_standardscripts: String,
    ts_wep: String,
}

impl AvroSchema for Summary {
    fn get_avro_schema(&self) -> String {
        r#"{"namespace": "lsst.obsenv","type": "record","name": "summary","fields": [{"name": "timestamp", "type": "long"},{"name": "spectractor", "type": "string"},{"name": "atmospec", "type": "string"},{"name": "cwfs", "type": "string"},{"name": "summit_extras", "type": "string"},{"name": "summit_utils", "type": "string"},{"name": "ts_config_attcs", "type": "string"},{"name": "ts_config_mttcs", "type": "string"},{"name": "ts_config_ocs", "type": "string"},{"name": "ts_config_scheduler", "type": "string"},{"name": "ts_externalscripts", "type": "string"},{"name": "ts_observatory_control", "type": "string"},{"name": "ts_observing_utilities", "type": "string"},{"name": "ts_standardscripts", "type": "string"},{"name": "ts_maintel_standardscripts", "type": "string"},{"name": "ts_auxtel_standardscripts", "type": "string"},{"name": "ts_wep", "type": "string"}]}"#
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
        let spectractor = extract_value!("Spectractor", summary);
        let atmospec = extract_value!("atmospec", summary);
        let cwfs = extract_value!("cwfs", summary);
        let summit_extras = extract_value!("summit_extras", summary);
        let summit_utils = extract_value!("summit_utils", summary);
        let ts_config_attcs = extract_value!("ts_config_attcs", summary);
        let ts_config_mttcs = extract_value!("ts_config_mttcs", summary);
        let ts_config_ocs = extract_value!("ts_config_ocs", summary);
        let ts_config_scheduler = extract_value!("ts_config_scheduler", summary);
        let ts_externalscripts = extract_value!("ts_externalscripts", summary);
        let ts_observatory_control = extract_value!("ts_observatory_control", summary);
        let ts_observing_utilities = extract_value!("ts_observing_utilities", summary);
        let ts_standardscripts = extract_value!("ts_standardscripts", summary);
        let ts_maintel_standardscripts = extract_value!("ts_maintel_standardscripts", summary);
        let ts_auxtel_standardscripts = extract_value!("ts_auxtel_standardscripts", summary);
        let ts_wep = extract_value!("ts_wep", summary);

        Summary {
            timestamp,
            spectractor,
            atmospec,
            cwfs,
            summit_extras,
            summit_utils,
            ts_config_attcs,
            ts_config_mttcs,
            ts_config_ocs,
            ts_config_scheduler,
            ts_externalscripts,
            ts_observatory_control,
            ts_observing_utilities,
            ts_standardscripts,
            ts_maintel_standardscripts,
            ts_auxtel_standardscripts,
            ts_wep,
        }
    }

    pub fn to_btree_map(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert("Spectractor".to_owned(), self.spectractor.clone());
        map.insert("atmospec".to_owned(), self.atmospec.clone());
        map.insert("cwfs".to_owned(), self.cwfs.clone());
        map.insert("summit_extras".to_owned(), self.summit_extras.clone());
        map.insert("summit_utils".to_owned(), self.summit_utils.clone());
        map.insert("ts_config_attcs".to_owned(), self.ts_config_attcs.clone());
        map.insert("ts_config_mttcs".to_owned(), self.ts_config_mttcs.clone());
        map.insert("ts_config_ocs".to_owned(), self.ts_config_ocs.clone());
        map.insert(
            "ts_config_scheduler".to_owned(),
            self.ts_config_scheduler.clone(),
        );
        map.insert(
            "ts_externalscripts".to_owned(),
            self.ts_externalscripts.clone(),
        );
        map.insert(
            "ts_observatory_control".to_owned(),
            self.ts_observatory_control.clone(),
        );
        map.insert(
            "ts_observing_utilities".to_owned(),
            self.ts_observing_utilities.clone(),
        );
        map.insert(
            "ts_standardscripts".to_owned(),
            self.ts_standardscripts.clone(),
        );
        map.insert(
            "ts_maintel_standardscripts".to_owned(),
            self.ts_maintel_standardscripts.clone(),
        );
        map.insert(
            "ts_auxtel_standardscripts".to_owned(),
            self.ts_auxtel_standardscripts.clone(),
        );
        map.insert("ts_wep".to_owned(), self.ts_wep.clone());
        map
    }

    pub fn get_topic_name() -> &'static str {
        "summary"
    }

    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn with_utc_now_timestamp(mut self) -> Self {
        self.timestamp = Utc::now().timestamp_millis();
        self
    }

    pub fn with_spectractor(mut self, spectractor: &str) -> Self {
        self.spectractor = spectractor.to_owned();
        self
    }

    pub fn with_atmospec(mut self, atmospec: &str) -> Self {
        self.atmospec = atmospec.to_owned();
        self
    }

    pub fn with_cwfs(mut self, cwfs: &str) -> Self {
        self.cwfs = cwfs.to_owned();
        self
    }

    pub fn with_summit_extras(mut self, summit_extras: &str) -> Self {
        self.summit_extras = summit_extras.to_owned();
        self
    }

    pub fn with_summit_utils(mut self, summit_utils: &str) -> Self {
        self.summit_utils = summit_utils.to_owned();
        self
    }

    pub fn with_ts_config_attcs(mut self, ts_config_attcs: &str) -> Self {
        self.ts_config_attcs = ts_config_attcs.to_owned();
        self
    }

    pub fn with_ts_config_mttcs(mut self, ts_config_mttcs: &str) -> Self {
        self.ts_config_mttcs = ts_config_mttcs.to_owned();
        self
    }

    pub fn with_ts_config_ocs(mut self, ts_config_ocs: &str) -> Self {
        self.ts_config_ocs = ts_config_ocs.to_owned();
        self
    }

    pub fn with_ts_config_scheduler(mut self, ts_config_scheduler: &str) -> Self {
        self.ts_config_scheduler = ts_config_scheduler.to_owned();
        self
    }

    pub fn with_ts_externalscripts(mut self, ts_externalscripts: &str) -> Self {
        self.ts_externalscripts = ts_externalscripts.to_owned();
        self
    }

    pub fn with_ts_observatory_control(mut self, ts_observatory_control: &str) -> Self {
        self.ts_observatory_control = ts_observatory_control.to_owned();
        self
    }

    pub fn with_ts_observing_utilities(mut self, ts_observing_utilities: &str) -> Self {
        self.ts_observing_utilities = ts_observing_utilities.to_owned();
        self
    }

    pub fn with_ts_standardscripts(mut self, ts_standardscripts: &str) -> Self {
        self.ts_standardscripts = ts_standardscripts.to_owned();
        self
    }

    pub fn with_ts_maintel_standardscripts(mut self, ts_maintel_standardscripts: &str) -> Self {
        self.ts_maintel_standardscripts = ts_maintel_standardscripts.to_owned();
        self
    }

    pub fn with_ts_auxtel_standardscripts(mut self, ts_auxtel_standardscripts: &str) -> Self {
        self.ts_auxtel_standardscripts = ts_auxtel_standardscripts.to_owned();
        self
    }

    pub fn with_ts_wep(mut self, ts_wep: &str) -> Self {
        self.ts_wep = ts_wep.to_owned();
        self
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
