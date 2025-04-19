//! # Qcloud Lighthouse
//!
//! Begin with the [`go`] function

use std::{borrow::Borrow, result::Result::Ok};

use anyhow::Result;
use hashbrown::HashSet;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::util::{
    BasicRequest, CommonResponse, Empty, MachineType, Secret, parse_response,
    request_builder,
};

/// Lighthouse instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRuleInfo {
    #[serde(rename = "AppType", skip_serializing_if = "Option::is_none")]
    pub app_type: Option<String>,

    #[serde(rename = "Protocol")]
    pub protocol: String,

    #[serde(rename = "Port")]
    pub port: String,

    #[serde(rename = "CidrBlock")]
    pub cidr_block: String,

    #[serde(rename = "Action")]
    pub action: String,

    #[serde(rename = "FirewallRuleDescription")]
    pub firewall_rule_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DescribeFirewallRulesRequest {
    #[serde(rename = "InstanceId")]
    instance_id: String,
    #[serde(rename = "Offset")]
    offset: i32,
    #[serde(rename = "Limit")]
    limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeFirewallRulesResponse {
    #[serde(rename = "FirewallRuleSet")]
    pub firewall_rule_set: Vec<FirewallRuleInfo>,
    #[serde(rename = "FirewallVersion")]
    pub firewall_version: i32,
    #[serde(rename = "TotalCount")]
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModifyFirewallRulesRequest {
    #[serde(rename = "InstanceId")]
    instance_id: String,
    #[serde(rename = "FirewallRules")]
    firewall_rules: Vec<FirewallRuleInfo>,
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
/// use solo_lib::sdk::qcloud::{
///     Secret,
///     lighthouse::{Instance, go},
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
    current_ip: &str,
    matched_descriptions: &[String],
) -> Result<()> {
    let response =
        list_rules(client, instance.borrow(), secret.borrow()).await?;
    let firewall_rules = &response.response.data.firewall_rule_set;
    let (firewall_rules, require_update) =
        compare_rules(firewall_rules, current_ip, matched_descriptions);
    if require_update {
        modify_rules(
            client,
            instance.borrow(),
            secret.borrow(),
            &firewall_rules,
        )
        .await?;
    }
    Ok(())
}

/// ### SDK Implementation DescribeFirewallRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn list_rules(
    client: &Client,
    instance: impl Borrow<Instance>,
    secret: impl Borrow<Secret>,
) -> Result<CommonResponse<DescribeFirewallRulesResponse>> {
    let instance = instance.borrow();

    let request = DescribeFirewallRulesRequest {
        instance_id: instance.id.clone(),
        offset: 0,
        limit: 100,
    };
    let payload = serde_json::to_string(&request)?;
    let basic_request = BasicRequest {
        machine_type: MachineType::Lighthouse,
        action: "DescribeFirewallRules",
        payload,
        region: instance.region.clone(),
        secret: secret.borrow(),
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<DescribeFirewallRulesResponse>>(&result)
}

/// ### SDK Process CompareRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub fn compare_rules(
    firewall_rules: &[FirewallRuleInfo],
    current_ip: &str,
    matched_descriptions: &[String],
) -> (Vec<FirewallRuleInfo>, bool) {
    let matched_set: HashSet<&str> =
        matched_descriptions.iter().map(|s| s.as_str()).collect();

    let mut require_update = false;
    let modified_rules: Vec<FirewallRuleInfo> = firewall_rules
        .iter()
        .map(|rule| {
            if matched_set.contains(rule.firewall_rule_description.as_str()) {
                if rule.cidr_block != current_ip {
                    require_update = true;
                    FirewallRuleInfo {
                        cidr_block: current_ip.into(),
                        ..rule.clone()
                    }
                } else {
                    rule.clone()
                }
            } else {
                rule.clone()
            }
        })
        .collect();

    (modified_rules, require_update)
}

/// ### SDK Implementation ModifyFirewallRules
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn modify_rules(
    client: &Client,
    instance: impl Borrow<Instance>,
    secret: impl Borrow<Secret>,
    firewall_rules: &[FirewallRuleInfo],
) -> Result<CommonResponse<Empty>> {
    let instance = instance.borrow();

    let firewall_rules: Vec<FirewallRuleInfo> = firewall_rules
        .iter()
        .cloned()
        .map(|mut rule| {
            rule.app_type = None;
            rule
        })
        .collect();

    let request = ModifyFirewallRulesRequest {
        instance_id: instance.id.clone(),
        firewall_rules,
    };

    let payload = serde_json::to_string(&request)?;
    let basic_request = BasicRequest {
        machine_type: MachineType::Lighthouse,
        action: "ModifyFirewallRules",
        payload,
        region: instance.region.clone(),
        secret: secret.borrow(),
    };

    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<Empty>>(&result)
}
