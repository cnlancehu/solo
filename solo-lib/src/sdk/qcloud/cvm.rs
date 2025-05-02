//! # Qcloud CVM
//!
//! Begin with the [`go`] function

use std::{borrow::Borrow, result::Result::Ok};

use anyhow::Result;
use hashbrown::HashSet;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::util::{CommonResponse, Empty, Secret, parse_response};
use crate::sdk::qcloud::util::{BasicRequest, MachineType, request_builder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroup {
    pub id: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroupPolicySet {
    #[serde(rename = "Version", skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(rename = "Egress", skip_serializing_if = "Vec::is_empty")]
    pub egress: Vec<SecurityGroupPolicy>,
    #[serde(rename = "Ingress", skip_serializing_if = "Vec::is_empty")]
    pub ingress: Vec<SecurityGroupPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroupPolicy {
    #[serde(rename = "PolicyIndex")]
    pub policy_index: i32,
    #[serde(rename = "Protocol", skip_serializing_if = "String::is_empty")]
    pub protocol: String,
    #[serde(rename = "Port", skip_serializing_if = "String::is_empty")]
    pub port: String,
    #[serde(
        rename = "ServiceTemplate",
        skip_serializing_if = "ServiceTemplateSpecification::is_empty"
    )]
    pub service_template: ServiceTemplateSpecification,
    #[serde(rename = "CidrBlock", skip_serializing_if = "String::is_empty")]
    pub cidr_block: String,
    #[serde(
        rename = "Ipv6CidrBlock",
        skip_serializing_if = "String::is_empty"
    )]
    pub ipv6_cidr_block: String,
    #[serde(
        rename = "SecurityGroupId",
        skip_serializing_if = "String::is_empty"
    )]
    pub security_group_id: String,
    #[serde(
        rename = "AddressTemplate",
        skip_serializing_if = "AddressTemplateSpecification::is_empty"
    )]
    pub address_template: AddressTemplateSpecification,
    #[serde(rename = "Action", skip_serializing_if = "String::is_empty")]
    pub action: String,
    #[serde(
        rename = "PolicyDescription",
        skip_serializing_if = "String::is_empty"
    )]
    pub policy_description: String,
    #[serde(rename = "ModifyTime", skip_serializing_if = "String::is_empty")]
    pub modify_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressTemplateSpecification {
    #[serde(rename = "AddressId", skip_serializing_if = "String::is_empty")]
    pub address_id: String,
    #[serde(
        rename = "AddressGroupId",
        skip_serializing_if = "String::is_empty"
    )]
    pub address_group_id: String,
}

impl AddressTemplateSpecification {
    pub fn is_empty(&self) -> bool {
        self.address_id.is_empty() && self.address_group_id.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTemplateSpecification {
    #[serde(rename = "ServiceId", skip_serializing_if = "String::is_empty")]
    pub service_id: String,
    #[serde(
        rename = "ServiceGroupId",
        skip_serializing_if = "String::is_empty"
    )]
    pub service_group_id: String,
}

impl ServiceTemplateSpecification {
    pub fn is_empty(&self) -> bool {
        self.service_id.is_empty() && self.service_group_id.is_empty()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DescribeSecurityGroupPoliciesRequest {
    #[serde(rename = "SecurityGroupId")]
    pub security_group_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DescribeSecurityGroupPoliciesResponse {
    #[serde(rename = "SecurityGroupPolicySet")]
    pub security_group_policy_set: SecurityGroupPolicySet,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplaceSecurityGroupPoliciesRequest {
    #[serde(rename = "SecurityGroupId")]
    pub security_group_id: String,
    #[serde(rename = "SecurityGroupPolicySet")]
    pub security_group_policy_set: SecurityGroupPolicySet,
}

/// ### Solo GO! - Main function
///
/// Start to modify the security group rules.
///
/// This function is a basic implementation of the SDK. It is recommended to use
/// this function directly.
///
/// ### Example
/// ```rust
/// use solo_lib::sdk::qcloud::{
///     Secret,
///     cvm::{SecurityGroup, go},
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
    let security_group_policy_set =
        response.response.data.security_group_policy_set;
    let (security_group_policy_set, require_update) = compare_rules(
        &security_group_policy_set,
        current_ipv4,
        current_ipv6,
        matched_descriptions,
    );
    if require_update {
        modify_rules(
            client,
            security_group.borrow(),
            secret.borrow(),
            &security_group_policy_set,
        )
        .await?;
    }
    Ok(())
}

/// ### SDK Implementation DescribeSecurityGroupPolicies
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn list_rules(
    client: &Client,
    security_group: impl Borrow<SecurityGroup>,
    secret: impl Borrow<Secret>,
) -> Result<CommonResponse<DescribeSecurityGroupPoliciesResponse>> {
    let security_group = security_group.borrow();

    let request = DescribeSecurityGroupPoliciesRequest {
        security_group_id: security_group.id.clone(),
    };
    let payload = serde_json::to_string(&request)?;
    let basic_request = BasicRequest {
        machine_type: MachineType::Cvm,
        action: "DescribeSecurityGroupPolicies",
        payload,
        region: security_group.region.clone(),
        secret: secret.borrow(),
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<DescribeSecurityGroupPoliciesResponse>>(
        &result,
    )
}

/// ### SDK Process CompareSecurityGroupPolicies
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub fn compare_rules(
    security_group_policy_set: &SecurityGroupPolicySet,
    current_ipv4: &str,
    current_ipv6: &str,
    matched_descriptions: &[String],
) -> (SecurityGroupPolicySet, bool) {
    let mut require_update = false;
    let matched_set: HashSet<_> =
        matched_descriptions.iter().map(|s| s.as_str()).collect();

    let mut ingress = security_group_policy_set.ingress.clone();
    for policy in &mut ingress {
        if !matched_set.contains(policy.policy_description.as_str()) {
            continue;
        }
        if policy.cidr_block.is_empty() {
            if !current_ipv6.is_empty()
                && policy.ipv6_cidr_block != current_ipv6
            {
                policy.ipv6_cidr_block = current_ipv6.to_string();
                require_update = true;
            }
        } else if policy.cidr_block != current_ipv4 {
            policy.cidr_block = current_ipv4.to_string();
            require_update = true;
        }
    }

    (
        SecurityGroupPolicySet {
            version: security_group_policy_set.version.clone(),
            egress: security_group_policy_set.egress.clone(),
            ingress,
        },
        require_update,
    )
}

/// ### SDK Implementation ReplaceSecurityGroupPolicies
///
/// Note that this function is a single step of solo. Use it only if you
/// would like to hook.
pub async fn modify_rules(
    client: &Client,
    security_group: impl Borrow<SecurityGroup>,
    secret: impl Borrow<Secret>,
    security_group_policy_set: &SecurityGroupPolicySet,
) -> Result<CommonResponse<Empty>> {
    let security_group = security_group.borrow();

    let security_group_policy_set = SecurityGroupPolicySet {
        egress: Vec::new(),
        ..security_group_policy_set.clone()
    };
    let request = ReplaceSecurityGroupPoliciesRequest {
        security_group_id: security_group.id.clone(),
        security_group_policy_set,
    };
    let payload = serde_json::to_string(&request)?;
    let basic_request = BasicRequest {
        machine_type: MachineType::Cvm,
        action: "ReplaceSecurityGroupPolicies",
        payload,
        region: security_group.region.clone(),
        secret: secret.borrow(),
    };
    let request = request_builder(client, basic_request)?;
    let result = client.execute(request).await?;
    let result = result.text().await?;

    parse_response::<CommonResponse<Empty>>(&result)
}
