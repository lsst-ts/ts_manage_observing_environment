use reqwest;
use serde_json;
use std::error::Error as StdError;

#[derive(Debug, Deserialize, Serialize, Default)]
struct KafkaClusterList {
    kind: String,
    metadata: Metadata,
    data: Vec<Data>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Metadata {
    #[serde(alias = "self")]
    url: String,
    next: Option<String>,
    resource_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Data {
    kind: String,
    metadata: Metadata,
    cluster_id: String,
    controller: Related,
    brokers: Related,
    broker_configs: Related,
    consumer_groups: Related,
    topics: Related,
    partition_reassignments: Related,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Related {
    related: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct TopicConfig {
    topic_name: String,
    partitions_count: usize,
    replication_factor: usize,
}

impl KafkaClusterList {
    fn get_cluster_id(&self) -> &str {
        &self.data[0].cluster_id
    }
}

impl TopicConfig {
    pub fn with_topic_name(mut self, topic_name: &str) -> Self {
        self.topic_name = topic_name.to_owned();
        self
    }

    pub fn with_partitions_count(mut self, partitions_count: usize) -> Self {
        self.partitions_count = partitions_count;
        self
    }

    pub fn with_replication_factor(mut self, replication_factor: usize) -> Self {
        self.replication_factor = replication_factor;
        self
    }
}

pub fn create_topics(sasquatch_rest_proxy_url: &str) -> Result<(), Box<dyn StdError>> {
    let client = reqwest::blocking::Client::new();
    let body = client
        .get(format!(
            "{sasquatch_rest_proxy_url}/sasquatch-rest-proxy/v3/clusters"
        ))
        .header("content-type", "application/json")
        .send()?
        .text()?;
    let kafka_cluster_list: KafkaClusterList = serde_json::from_str(&body)?;
    let cluster_id = kafka_cluster_list.get_cluster_id();
    let topic_config = TopicConfig::default()
        .with_topic_name("lsst.obsenv.summary")
        .with_partitions_count(1)
        .with_replication_factor(3);
    log::debug!("{topic_config:?}");
    let res = client
        .post(format!(
            "{sasquatch_rest_proxy_url}/sasquatch-rest-proxy/v3/clusters/{cluster_id}/topics"
        ))
        .json(&topic_config)
        .send()?;
    log::debug!("{res:?}");
    let topic_config = TopicConfig::default()
        .with_topic_name("lsst.obsenv.action")
        .with_partitions_count(1)
        .with_replication_factor(3);
    log::debug!("{topic_config:?}");
    let res = client
        .post(format!(
            "{sasquatch_rest_proxy_url}/sasquatch-rest-proxy/v3/clusters/{cluster_id}/topics"
        ))
        .json(&topic_config)
        .send()?;
    log::debug!("{res:?}");
    Ok(())
}
