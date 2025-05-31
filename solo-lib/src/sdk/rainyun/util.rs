use anyhow::Result;
use http::Method;
use reqwest::{Client, Request};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};

use crate::SdkError;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommonResponse<T> {
    pub code: u32,
    pub data: T,
}

#[derive(Debug, Clone)]
pub(super) struct BasicRequest<'a> {
    pub(super) payload: String,
    pub(super) method: Method,

    pub(super) query: Option<String>,
    pub(super) instance_id: &'a str,
    pub(super) token: &'a str,
}

pub(super) fn request_builder(
    client: &Client,
    basic_request: BasicRequest,
) -> Result<Request> {
    let endpoint = format!(
        "https://api.v2.rainyun.com/product/rcs/{}/firewall/rule{}",
        basic_request.instance_id,
        basic_request.query.unwrap_or_default()
    );

    Ok(client
        .request(basic_request.method, endpoint)
        .header("x-api-key", basic_request.token)
        .body(basic_request.payload)
        .build()?)
}

fn to_error_response(response: &str) -> Option<SdkError> {
    let response: Value = if let Ok(response) = from_str(response) {
        response
    } else {
        return None;
    };
    let code = response.get("code")?.as_u64()?;
    let message = response.get("message")?.as_str()?.to_string();
    if code != 200 {
        Some(SdkError {
            code: code.to_string(),
            message,
            request_id: String::new(),
        })
    } else {
        None
    }
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
