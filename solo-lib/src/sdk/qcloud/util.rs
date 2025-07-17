use std::{result::Result::Ok, time::SystemTime};

use anyhow::{Error, Result};
use chrono::Utc;
use hmac::{Hmac, Mac};
use http::HeaderMap;
use reqwest::{Client, Request};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use sha2::{Digest, Sha256};

use crate::{
    error::SdkError,
    util::{hmac256, sha256_hex},
};

/// Secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub secret_id: String,
    pub secret_key: String,
}

/// Machine Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MachineType {
    /// Lighthouse instance
    Lighthouse,

    /// Cvm instance
    Cvm,
}

impl MachineType {
    /// Returns (service, host, version, endpoint)
    fn service_info(
        &self,
    ) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            MachineType::Lighthouse => (
                "lighthouse",
                "lighthouse.tencentcloudapi.com",
                "2020-03-24",
                "https://lighthouse.tencentcloudapi.com",
            ),
            MachineType::Cvm => (
                "vpc",
                "vpc.tencentcloudapi.com",
                "2017-03-12",
                "https://vpc.tencentcloudapi.com",
            ),
        }
    }
}

/// Common response
///
/// The response from the API is wrapped in this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonResponse<T> {
    #[serde(rename = "Response")]
    pub response: ResponseWrapper<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseWrapper<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(rename = "RequestId")]
    pub request_id: String,
}

/// An empty struct, used for requests that don't require a payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "Error")]
    pub error: SdkError,
}

#[derive(Debug, Clone)]
pub(super) struct BasicRequest<'a> {
    pub(super) machine_type: MachineType,
    pub(super) action: &'static str, // 改为静态字符串
    pub(super) payload: String,
    pub(super) region: String,
    pub(super) secret: &'a Secret,
}

pub(super) fn request_builder(
    client: &Client,
    basic_request: BasicRequest,
) -> Result<Request> {
    let (service, host, version, endpoint) =
        basic_request.machine_type.service_info();

    let algorithm = "TC3-HMAC-SHA256";
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    let date = Utc::now().format("%Y-%m-%d").to_string();

    let http_request_method = "POST";
    let canonical_uri = "/";
    let canonical_querystring = "";
    let ct = "application/json";

    let canonical_headers = format!(
        "content-type:{}\nhost:{}\nx-tc-action:{}\n",
        ct,
        host,
        basic_request.action.to_lowercase()
    );

    let signed_headers = "content-type;host;x-tc-action";
    let hashed_request_payload = sha256_hex(&basic_request.payload);
    let canonical_request = format!(
        "{http_request_method}\n{canonical_uri}\n{canonical_querystring}\n{canonical_headers}\n{signed_headers}\n{hashed_request_payload}"
    );

    let credential_scope = format!("{date}/{service}/tc3_request");
    let hashed_canonical_request = {
        let mut hasher = Sha256::new();
        hasher.update(canonical_request.as_bytes());
        format!("{:x}", hasher.finalize())
    };
    let string_to_sign = format!(
        "{algorithm}\n{timestamp}\n{credential_scope}\n{hashed_canonical_request}"
    );

    let secret_date = sign(
        format!("TC3{}", basic_request.secret.secret_key).as_bytes(),
        &date,
    );
    let secret_service = sign(&secret_date, service);
    let secret_signing = sign(&secret_service, "tc3_request");
    let signature = hmac256(&secret_signing, &string_to_sign)
        .map_err(Error::msg)
        .map(hex::encode)?;

    let authorization = format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        algorithm,
        basic_request.secret.secret_id,
        credential_scope,
        signed_headers,
        signature
    );

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", authorization.parse()?);
    headers.insert("Content-Type", ct.parse()?);
    headers.insert("Host", host.parse()?);
    headers.insert("X-TC-Action", basic_request.action.parse()?);
    headers.insert("X-TC-Timestamp", timestamp.to_string().parse()?);
    headers.insert("X-TC-Version", version.parse()?);
    headers.insert("X-TC-Region", basic_request.region.parse()?);

    Ok(client
        .post(endpoint)
        .headers(headers)
        .body(basic_request.payload)
        .build()?)
}

fn to_error_response(response: &str) -> Option<SdkError> {
    let response: Value = if let Ok(response) = from_str(response) {
        response
    } else {
        return None;
    };
    let response = response.get("Response")?;
    let request_id = response.get("RequestId")?.as_str()?.to_string();
    let error = response.get("Error")?;
    let code = error.get("Code")?.as_str()?.to_string();
    let message = error.get("Message")?.as_str()?.to_string();
    Some(SdkError {
        request_id,
        code,
        message,
    })
}

pub(super) fn parse_response<'a, T: Deserialize<'a>>(
    result: &'a str,
) -> Result<T> {
    if let Some(error) = to_error_response(result) {
        Err(error.into())
    } else {
        Ok(from_str::<T>(result)?)
    }
}

pub(super) fn sign(key: &[u8], msg: &str) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(msg.as_bytes());
    mac.finalize().into_bytes().to_vec()
}
