//! # Aliyun SAS
//!
//! Begin with the [`go`] function

use std::{borrow::Borrow, collections::HashSet};

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use super::util::{
    BasicRequest, Empty, MachineType, Secret, parse_response, request_builder,
};
use crate::sdk::aliyun::util::CommonResponse;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    #[serde(rename = "Remark")]
    pub remark: String,
    #[serde(rename = "Port")]
    pub port: String,
    #[serde(rename = "RuleId")]
    pub rule_id: String,
    #[serde(rename = "RuleProtocol")]
    pub rule_protocol: String,
    #[serde(rename = "Policy")]
    pub policy: String,
    #[serde(rename = "SourceCidrIp")]
    pub source_cidr_ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFirewallRulesResponse {
    #[serde(rename = "TotalCount")]
    pub total_count: i32,
    #[serde(rename = "PageSize")]
    pub page_size: i32,
    #[serde(rename = "PageNumber")]
    pub page_number: i32,
    #[serde(rename = "FirewallRules")]
    pub firewall_rules: Vec<FirewallRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFirewallRulesResponse {
    #[serde(rename = "FirewallRuleIds")]
    pub firewall_rule_ids: Vec<String>,
}

/// ### Solo GO! - Main function
///
/// Start to modify the firewall rules.
///
/// This function is a basic implementation of the SDK. It is recommended to use
/// this function directly.
///
/// ### Example
/// ```rust
/// use solo_lib::sdk::aliyun::{
///     Secret,
///     sas::{Instance, go},
/// };
///
/// #[tokio::main]
/// async fn main() {
///     let client = solo_lib::client::new();
///     let secret = Secret {
///         secret_id: "secret_id".to_string(),
///         secret_key: "secret_key".to_string(),
///     };
///     let instance = Instance {
///         id: "instance_id".to_string(),
///         region: "instance_region".to_string(),
///     };
///     let _result = go(
///         &client,
///         &instance,
///         &secret,
///         "current_ipv4",
///         &[
///             "firewall_rule_one".to_string(),
///             "firewall_rule_two".to_string(),
///         ],
///     )
///     .await
///     .unwrap();
/// }
/// ```
pub async fn go(
    client: &Client,
    instance: impl Borrow<Instance>,
    secret: impl Borrow<Secret>,
    current_ipv4: &str,
    matched_descriptions: &[String],
) -> Result<()> {
    let response =
        list_rules(client, instance.borrow(), secret.borrow()).await?;
    let firewall_rules = response.response.firewall_rules;
    let (firewall_rules, require_update) =
        compare_rules(&firewall_rules, current_ipv4, matched_descriptions);
    if require_update {
        delete_rules(
            client,
            instance.borrow(),
            secret.borrow(),
            &firewall_rules,
        )
        .await?;
        create_rules(
            client,
            instance.borrow(),
            secret.borrow(),
            &firewall_rules,
        )
        .await?;
    }
    Ok(())
}

/// ### SDK Implementation ListFirewallRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn list_rules(
    client: &Client,
    instance: impl Borrow<Instance>,
    secret: impl Borrow<Secret>,
) -> Result<CommonResponse<ListFirewallRulesResponse>> {
    let instance = instance.borrow();

    let params = vec![
        ("InstanceId", instance.id.as_str()),
        ("RegionId", instance.region.as_str()),
        ("PageSize", "100"),
    ];
    let basic_request = BasicRequest {
        machine_type: MachineType::Sas,
        region_id: instance.region.as_str(),
        action: "ListFirewallRules",
        secret: secret.borrow(),
        params: &params,
        body: "",
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<ListFirewallRulesResponse>>(&result)
}

/// ### SDK Process CompareRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub fn compare_rules(
    firewall_rules: &[FirewallRule],
    current_ip: &str,
    matched_descriptions: &[String],
) -> (Vec<FirewallRule>, bool) {
    let matched_set: HashSet<String> =
        matched_descriptions.iter().cloned().collect();

    let mut require_update = false;
    let mut firewall_rules_to_be_modified: Vec<FirewallRule> = Vec::new();

    firewall_rules.iter().for_each(|rule| {
        if matched_set.contains(rule.remark.as_str())
            && rule.source_cidr_ip != current_ip
        {
            let mut rule = rule.clone();
            rule.source_cidr_ip = current_ip.into();
            firewall_rules_to_be_modified.push(rule);
            require_update = true;
        }
    });
    (firewall_rules_to_be_modified, require_update)
}

/// ### SDK Implementation DeleteFirewallRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn delete_rules(
    client: &Client,
    instance: impl Borrow<Instance>,
    secret: impl Borrow<Secret>,
    firewall_rules: &[FirewallRule],
) -> Result<CommonResponse<Empty>> {
    let instance = instance.borrow();

    let rule_ids = firewall_rules
        .iter()
        .map(|rule| rule.rule_id.as_str()) // 直接引用
        .collect::<Vec<_>>()
        .join(",");

    let params = vec![
        ("InstanceId", instance.id.as_str()),
        ("RegionId", instance.region.as_str()),
        ("RuleIds", rule_ids.as_str()),
    ];
    let basic_request = BasicRequest {
        machine_type: MachineType::Sas,
        region_id: instance.region.as_str(),
        action: "DeleteFirewallRules",
        secret: secret.borrow(),
        params: &params,
        body: "",
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<Empty>>(&result)
}

/// ### SDK Implementation CreateFirewallRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn create_rules(
    client: &Client,
    instance: impl Borrow<Instance>,
    secret: impl Borrow<Secret>,
    firewall_rules: &[FirewallRule],
) -> Result<CommonResponse<CreateFirewallRulesResponse>> {
    let instance = instance.borrow();

    let rules = to_string(firewall_rules)?;
    let params = vec![
        ("InstanceId", instance.id.as_str()),
        ("RegionId", instance.region.as_str()),
        ("FirewallRules", &rules),
    ];
    let basic_request = BasicRequest {
        machine_type: MachineType::Sas,
        region_id: instance.region.as_str(),
        action: "CreateFirewallRules",
        secret: secret.borrow(),
        params: &params,
        body: "",
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<CreateFirewallRulesResponse>>(&result)
}
