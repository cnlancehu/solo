//! # Aliyun ECS
//!
//! Begin with the [`go`] function

use std::{borrow::Borrow, collections::HashSet};

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::to_value;

use super::util::{
    BasicRequest, CommonResponse, Empty, MachineType, Secret, flatten_json,
    parse_response, request_builder,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroup {
    pub id: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroupRule {
    #[serde(
        rename = "SecurityGroupRuleId",
        skip_serializing_if = "String::is_empty"
    )]
    pub security_group_rule_id: String,

    #[serde(rename = "Direction", skip_serializing_if = "String::is_empty")]
    pub direction: String,

    #[serde(
        rename = "SourceGroupId",
        skip_serializing_if = "String::is_empty"
    )]
    pub source_group_id: String,

    #[serde(
        rename = "DestGroupOwnerAccount",
        skip_serializing_if = "String::is_empty"
    )]
    pub dest_group_owner_account: String,

    #[serde(
        rename = "DestPrefixListId",
        skip_serializing_if = "String::is_empty"
    )]
    pub dest_prefix_list_id: String,

    #[serde(
        rename = "DestPrefixListName",
        skip_serializing_if = "String::is_empty"
    )]
    pub dest_prefix_list_name: String,

    #[serde(rename = "SourceCidrIp", skip_serializing_if = "String::is_empty")]
    pub source_cidr_ip: String,

    #[serde(
        rename = "Ipv6DestCidrIp",
        skip_serializing_if = "String::is_empty"
    )]
    pub ipv6_dest_cidr_ip: String,

    #[serde(rename = "CreateTime", skip_serializing_if = "String::is_empty")]
    pub create_time: String,

    #[serde(
        rename = "Ipv6SourceCidrIp",
        skip_serializing_if = "String::is_empty"
    )]
    pub ipv6_source_cidr_ip: String,

    #[serde(rename = "DestGroupId", skip_serializing_if = "String::is_empty")]
    pub dest_group_id: String,

    #[serde(rename = "DestCidrIp", skip_serializing_if = "String::is_empty")]
    pub dest_cidr_ip: String,

    #[serde(rename = "IpProtocol", skip_serializing_if = "String::is_empty")]
    pub ip_protocol: String,

    #[serde(rename = "Priority")]
    pub priority: i32,

    #[serde(
        rename = "DestGroupName",
        skip_serializing_if = "String::is_empty"
    )]
    pub dest_group_name: String,

    #[serde(rename = "NicType", skip_serializing_if = "String::is_empty")]
    pub nic_type: String,

    #[serde(rename = "Policy", skip_serializing_if = "String::is_empty")]
    pub policy: String,

    #[serde(rename = "Description", skip_serializing_if = "String::is_empty")]
    pub description: String,

    #[serde(rename = "PortRange", skip_serializing_if = "String::is_empty")]
    pub port_range: String,

    #[serde(
        rename = "SourcePrefixListName",
        skip_serializing_if = "String::is_empty"
    )]
    pub source_prefix_list_name: String,

    #[serde(
        rename = "SourcePrefixListId",
        skip_serializing_if = "String::is_empty"
    )]
    pub source_prefix_list_id: String,

    #[serde(
        rename = "SourceGroupOwnerAccount",
        skip_serializing_if = "String::is_empty"
    )]
    pub source_group_owner_account: String,

    #[serde(
        rename = "SourceGroupName",
        skip_serializing_if = "String::is_empty"
    )]
    pub source_group_name: String,

    #[serde(
        rename = "SourcePortRange",
        skip_serializing_if = "String::is_empty"
    )]
    pub source_port_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeSecurityGroupAttributeResponse {
    #[serde(rename = "Permissions")]
    pub permissions: DescribeSecurityGroupAttributeResponsePermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeSecurityGroupAttributeResponsePermissions {
    #[serde(rename = "Permission")]
    pub permission: Vec<SecurityGroupRule>,
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
///     ecs::{SecurityGroup, go},
/// };
///
/// #[tokio::main]
/// async fn main() {
///     let client = solo_lib::client::new();
///     let secret = Secret {
///         secret_id: "secret_id".to_string(),
///         secret_key: "secret_key".to_string(),
///     };
///     let security_group = SecurityGroup {
///         id: "security_group_id".to_string(),
///         region: "security_group_region".to_string(),
///     };
///     let _result = go(
///         &client,
///         &security_group,
///         &secret,
///         "current_ipv4",
///         "current_ipv6",
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
    security_group: impl Borrow<SecurityGroup>,
    secret: impl Borrow<Secret>,
    current_ipv4: &str,
    current_ipv6: &str,
    matched_descriptions: &[String],
) -> Result<()> {
    let response =
        list_rules(client, security_group.borrow(), secret.borrow()).await?;
    let security_group_rules = &response.response.permissions.permission;
    let (firewall_rules_to_be_modified, require_update) = compare_rules(
        security_group_rules,
        current_ipv4,
        current_ipv6,
        matched_descriptions,
    );
    if require_update {
        delete_rules(
            client,
            security_group.borrow(),
            secret.borrow(),
            &firewall_rules_to_be_modified,
        )
        .await?;
        create_rules(
            client,
            security_group.borrow(),
            secret.borrow(),
            &firewall_rules_to_be_modified,
        )
        .await?;
    }
    Ok(())
}

/// ### SDK Implementation DescribeSecurityGroupAttribute
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn list_rules(
    client: &Client,
    security_group: &SecurityGroup,
    secret: &Secret,
) -> Result<CommonResponse<DescribeSecurityGroupAttributeResponse>> {
    let params = vec![
        ("SecurityGroupId", security_group.id.as_str()),
        ("RegionId", security_group.region.as_str()),
        ("Direction", "ingress"),
        ("MaxResults", "1000"),
    ];
    let basic_request = BasicRequest {
        machine_type: MachineType::Ecs,
        region_id: &security_group.region,
        action: "DescribeSecurityGroupAttribute",
        secret,
        params: &params,
        body: "",
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<DescribeSecurityGroupAttributeResponse>>(
        &result,
    )
}

/// ### SDK Process CompareSecurityGroupPolicies
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub fn compare_rules(
    security_group_rules: &[SecurityGroupRule],
    current_ipv4: &str,
    current_ipv6: &str,
    matched_descriptions: &[String],
) -> (Vec<SecurityGroupRule>, bool) {
    let matched_set: HashSet<String> =
        matched_descriptions.iter().cloned().collect();

    let mut require_update = false;
    let mut firewall_rules_to_be_modified: Vec<SecurityGroupRule> = Vec::new();

    security_group_rules.iter().for_each(|rule| {
        if matched_set.contains(rule.description.as_str()) {
            if !rule.ipv6_source_cidr_ip.is_empty()
                && rule.ipv6_source_cidr_ip != current_ipv6
            {
                firewall_rules_to_be_modified.push(SecurityGroupRule {
                    ipv6_source_cidr_ip: current_ipv6.into(),
                    ..rule.clone()
                });
                require_update = true;
            } else if rule.source_cidr_ip != current_ipv4 {
                firewall_rules_to_be_modified.push(SecurityGroupRule {
                    source_cidr_ip: current_ipv4.into(),
                    ..rule.clone()
                });
                require_update = true;
            }
        }
    });

    (firewall_rules_to_be_modified, require_update)
}

/// ### SDK Implementation RevokeSecurityGroup
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn delete_rules(
    client: &Client,
    security_group: &SecurityGroup,
    secret: &Secret,
    security_group_rules: &[SecurityGroupRule],
) -> Result<CommonResponse<Empty>> {
    let rule_ids = security_group_rules
        .iter()
        .map(|rule| rule.security_group_rule_id.as_str())
        .collect::<Vec<_>>();
    let mut params = vec![
        ("SecurityGroupId", security_group.id.as_str()),
        ("RegionId", security_group.region.as_str()),
    ];
    let mut keys = Vec::new();
    for i in 0..rule_ids.len() {
        keys.push(format!("SecurityGroupRuleId.{}", i + 1));
    }

    for (key, value) in keys.iter().zip(rule_ids.iter()) {
        params.push((key.as_str(), *value));
    }
    let basic_request: BasicRequest<'_> = BasicRequest {
        machine_type: MachineType::Ecs,
        region_id: &security_group.region,
        action: "RevokeSecurityGroup",
        secret,
        params: &params,
        body: "",
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<Empty>>(&result)
}

/// ### SDK Implementation AuthorizeSecurityGroup
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn create_rules(
    client: &Client,
    security_group: &SecurityGroup,
    secret: &Secret,
    security_group_rules: &[SecurityGroupRule],
) -> Result<CommonResponse<Empty>> {
    let rules = flatten_json(&to_value(security_group_rules)?, "Permissions");
    let rules: Vec<(&str, &str)> =
        rules.iter().map(|(a, b)| (&**a, &**b)).collect();
    let mut params = vec![
        ("SecurityGroupId", security_group.id.as_str()),
        ("RegionId", security_group.region.as_str()),
    ];
    params.extend(rules);
    let basic_request = BasicRequest {
        machine_type: MachineType::Ecs,
        region_id: &security_group.region,
        action: "AuthorizeSecurityGroup",
        secret,
        params: &params,
        body: "",
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<Empty>>(&result)
}
