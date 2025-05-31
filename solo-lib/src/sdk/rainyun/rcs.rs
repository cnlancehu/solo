use anyhow::Result;
use hashbrown::HashSet;
use http::Method;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{CommonResponse, parse_response};
use crate::sdk::rainyun::{BasicRequest, request_builder};

#[derive(Deserialize, Debug, Clone)]
pub struct DescribeFirewallRulesResponse {
    #[serde(rename = "Records")]
    pub records: Vec<Record>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Record {
    #[serde(rename = "ID")]
    pub id: u32,
    pub is_enable: bool,
    pub pos: u32,
    pub source_address: String,
    pub dest_port: String,
    pub protocol: String,
    pub action: String,
    pub description: String,
}

pub async fn list_rules<'a>(
    client: &Client,
    instance_id: &'a str,
    token: &'a str,
) -> Result<CommonResponse<DescribeFirewallRulesResponse>> {
    let basic_request = BasicRequest {
        payload: String::new(),
        method: Method::GET,
        query: Some("?options=null".to_string()),

        instance_id,
        token,
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<DescribeFirewallRulesResponse>>(&result)
}

pub fn compare_rules(
    records: &[Record],
    current_ipv4: &str,
    matched_descriptions: &[String],
) -> (Vec<Record>, bool) {
    let matched_set: HashSet<_> =
        matched_descriptions.iter().map(|s| s.as_str()).collect();

    let mut require_update = false;
    let mut recordss_to_be_modified: Vec<Record> = Vec::new();

    records.iter().for_each(|record| {
        if matched_set.contains(record.description.as_str())
            && record.source_address != current_ipv4
        {
            let mut record = record.clone();
            record.source_address = current_ipv4.to_string();
            recordss_to_be_modified.push(record);
            require_update = true;
        }
    });

    (recordss_to_be_modified, require_update)
}

pub async fn modify_rules<'a>(
    client: &Client,
    instance_id: &'a str,
    token: &'a str,
    firewall_rules: &[Record],
) -> Result<()> {
    for rule in firewall_rules {
        let payload = serde_json::to_string(rule)?;

        let basic_request = BasicRequest {
            payload,
            method: Method::POST,
            query: None,

            instance_id,
            token,
        };
        let request = request_builder(client, basic_request)?;
        client.execute(request).await?;
    }
    Ok(())
}
