use core::str;
use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Result;
use chrono::DateTime;
use http::Method;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rand::Rng;
use reqwest::{
    Client, Request,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};

use crate::{
    error::SdkError,
    util::{current_timestamp, hmac256, sha256_hex},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub secret_id: String,
    pub secret_key: String,
}

/// Machine Type
#[derive(Debug, Clone)]
pub enum MachineType {
    /// SAS instance
    Sas,

    /// ECS instance
    Ecs,
}

impl MachineType {
    /// Returns (endpoint, version)
    pub fn service_info(&self, region_id: &str) -> (String, &'static str) {
        match self {
            MachineType::Sas => {
                (format!("swas.{}.aliyuncs.com", region_id), "2020-06-01")
            }
            MachineType::Ecs => {
                (format!("ecs.{}.aliyuncs.com", region_id), "2014-05-26")
            }
        }
    }
}

/// Common response
///
/// The response from the API is wrapped in this struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonResponse<T> {
    #[serde(rename = "RequestId")]
    pub request_id: String,
    #[serde(flatten)]
    pub response: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empty {}

#[derive(Debug, Clone)]
pub(super) struct BasicRequest<'a> {
    pub(super) machine_type: MachineType,
    pub(super) region_id: &'a str,
    pub(super) secret: &'a Secret,
    pub(super) action: &'static str,
    pub(super) params: &'a [(&'a str, &'a str)],
    pub(super) body: &'a str,
}

pub(super) fn request_builder(
    client: &Client,
    basic_request: BasicRequest<'_>,
) -> Result<Request> {
    let (host, version) = basic_request
        .machine_type
        .service_info(basic_request.region_id);
    let canonical_uri = "/";
    let canonical_query_string =
        build_sored_encoded_query_string(basic_request.params);
    let hashed_request_payload = sha256_hex("");
    let now_time = current_timestamp()?;
    let datetime = DateTime::from_timestamp(now_time as i64, 0).unwrap();
    let datetime_str = datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let signature_nonce = generate_nonce();

    // 构造请求头
    let mut headers = HeaderMap::new();
    headers.insert("Host", HeaderValue::from_str(&host)?);
    headers.insert(
        "Content-Type",
        HeaderValue::from_str("application/json; charset=utf-8")?,
    );
    headers
        .insert("x-acs-action", HeaderValue::from_str(basic_request.action)?);
    headers.insert("x-acs-version", HeaderValue::from_str(version)?);
    headers.insert("x-acs-date", HeaderValue::from_str(&datetime_str)?);
    headers.insert(
        "x-acs-signature-nonce",
        HeaderValue::from_str(&signature_nonce)?,
    );
    headers.insert(
        "x-acs-content-sha256",
        HeaderValue::from_str(&hashed_request_payload)?,
    );

    let sign_header_arr = &[
        "host",
        "x-acs-action",
        "x-acs-content-sha256",
        "x-acs-date",
        "x-acs-signature-nonce",
        "x-acs-version",
    ];

    let http_request_method = "POST";

    let sign_headers = sign_header_arr.join(";");
    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n\n{}\n{}",
        http_request_method,
        canonical_uri,
        canonical_query_string,
        sign_header_arr
            .iter()
            .map(|&header| format!(
                "{}:{}",
                header,
                headers[header].to_str().unwrap()
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        sign_headers,
        hashed_request_payload,
    );

    let result = sha256_hex(&canonical_request);
    let string_to_sign = format!("ACS3-HMAC-SHA256\n{}", result);
    let signature =
        hmac256(basic_request.secret.secret_key.as_bytes(), &string_to_sign)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    let data_sign = hex::encode(&signature);
    let auth_data = format!(
        "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
        basic_request.secret.secret_id, sign_headers, data_sign
    );

    headers.insert("Authorization", HeaderValue::from_str(&auth_data)?);

    let url = format!("https://{}{}", host, canonical_uri);
    Ok(client
        .request(Method::POST, url)
        .headers(headers)
        .query(&basic_request.params)
        .body(basic_request.body.to_string())
        .build()?)
}

fn to_error_response(response: &str) -> Option<SdkError> {
    let response: Value = if let Ok(response) = from_str(response) {
        response
    } else {
        return None;
    };
    let request_id = response.get("RequestId")?.as_str()?.to_string();
    let code = response.get("Code")?.as_str()?.to_string();
    let message = response.get("Message")?.as_str()?.to_string();
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

fn build_sored_encoded_query_string(query_params: &[(&str, &str)]) -> String {
    let sorted_query_params: BTreeMap<_, _> =
        query_params.iter().copied().collect();

    let encoded_params: Vec<String> = sorted_query_params
        .into_iter()
        .map(|(k, v)| {
            let encoded_key = percent_code(k);
            let encoded_value = percent_code(v);
            format!("{}={}", encoded_key, encoded_value)
        })
        .collect();

    encoded_params.join("&")
}

fn percent_code(encode_str: &str) -> Cow<'_, str> {
    utf8_percent_encode(encode_str, NON_ALPHANUMERIC)
        .collect::<String>()
        .replace("%5F", "_")
        .replace("%2D", "-")
        .replace("%2E", ".")
        .replace("%7E", "~")
        .into()
}

fn generate_nonce() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..32)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

pub(super) fn flatten_json(
    value: &Value,
    prefix: &str,
) -> Vec<(String, String)> {
    let mut result = Vec::new();

    fn process_value(
        value: &Value,
        current_prefix: &str,
        result: &mut Vec<(String, String)>,
    ) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_prefix = if current_prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", current_prefix, key)
                    };
                    process_value(val, &new_prefix, result);
                }
            }
            Value::Array(arr) => {
                for (index, elem) in arr.iter().enumerate() {
                    let new_prefix =
                        format!("{}.{}", current_prefix, index + 1);
                    process_value(elem, &new_prefix, result);
                }
            }
            Value::Null => {
                result.push((current_prefix.to_string(), "null".to_string()))
            }
            Value::Bool(b) => {
                result.push((current_prefix.to_string(), b.to_string()))
            }
            Value::Number(n) => {
                result.push((current_prefix.to_string(), n.to_string()))
            }
            Value::String(s) => {
                result.push((current_prefix.to_string(), s.clone()))
            }
        }
    }

    process_value(value, prefix, &mut result);
    result
}
