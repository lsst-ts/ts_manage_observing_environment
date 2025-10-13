use std::error::Error;

use super::log_summary::AvroSchema;
use chrono::Utc;
use lsst_efd_client::EfdAuth;
use reqwest::blocking::Client;
use thiserror::Error as ThisError;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RunBranch {
    timestamp: i64,
    branch_name: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct QueryResult<T> {
    pub results: Vec<Payload<T>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct RunBranchSeries {
    name: String,
    columns: Vec<String>,
    values: Vec<(String, i64, String)>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Payload<T> {
    statement_id: usize,
    pub series: Vec<T>,
}

#[derive(Clone, Debug, Eq, ThisError, PartialEq)]
#[error("{0}")]
struct ErrorRetrievingRunBranch(String);

impl RunBranchSeries {
    fn as_run_branch(&self) -> RunBranch {
        RunBranch {
            timestamp: self.values[0].1,
            branch_name: self.values[0].2.clone(),
        }
    }
}
impl AvroSchema for RunBranch {
    fn get_avro_schema(&self) -> String {
        r#"{"namespace": "lsst.obsenv","type": "record","name": "run_branch","fields": [{"name": "timestamp", "type": "long"},{"name": "branch_name", "type": "string"}]}"#.to_owned()
    }
}

impl RunBranch {
    pub fn new(branch_name: &str) -> RunBranch {
        RunBranch {
            timestamp: Utc::now().timestamp_millis(),
            branch_name: branch_name.to_owned(),
        }
    }

    pub fn get_topic_name() -> &'static str {
        "run_branch"
    }

    pub fn get_branch_name(&self) -> &str {
        &self.branch_name
    }

    pub fn retrieve_from_efd(efd_name: &str) -> Result<RunBranch, Box<dyn Error>> {
        let efd_auth = EfdAuth::new_blocking(efd_name)?;

        let influxdb_url = format!(
            "https://{}:{}/influxdb/query",
            efd_auth.get_host(),
            efd_auth.get_port(),
        );

        // Create a reqwest client
        let client = Client::new();

        let query = r#"SELECT "timestamp", "branch_name" FROM "lsst.obsenv"."autogen"."lsst.obsenv.run_branch" ORDER BY DESC LIMIT 1"#;

        // Construct the full URL with query parameters
        let response = client
            .get(influxdb_url)
            .basic_auth(efd_auth.get_username(), Some(efd_auth.get_password()))
            .query(&[("db", "efd"), ("q", query)])
            .send()?; // Check the status code

        if response.status().is_success() {
            // Parse the response JSON
            let text = response.text()?;
            let query_result: Result<QueryResult<RunBranchSeries>, serde_json::Error> =
                serde_json::from_str(&text);
            match query_result {
                Ok(query_result) => Ok(query_result.results[0].series[0].as_run_branch()),
                Err(error) => Err(Box::new(ErrorRetrievingRunBranch(format!(
                    "Error: {error:?} parsing response: {text:?}"
                )))),
            }
        } else {
            Err(Box::new(ErrorRetrievingRunBranch(format!(
                "Error: {:?}",
                response
            ))))
        }
    }
}
