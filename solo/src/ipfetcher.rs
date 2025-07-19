use std::{borrow::Cow, net::IpAddr, time::Duration};

use anyhow::Result;
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
/// The IP protocol version the server uses.
pub enum Protocol {
    #[serde(rename = "v4")]
    V4,
    #[serde(rename = "v6")]
    V6,
    #[serde(rename = "both")]
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpProvider {
    #[serde(rename = "url")]
    Url(String),
    #[serde(rename = "embed")]
    Embed(EmbedIpProvider),
}

impl Default for IpProvider {
    fn default() -> Self {
        Self::Embed(EmbedIpProvider::MyExternalIp)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbedIpProvider {
    #[serde(rename = "ipecho")]
    IpEcho,
    #[serde(rename = "curlmyip")]
    CurlMyIp,
    #[serde(rename = "myexternalip")]
    MyExternalIp,
}

impl EmbedIpProvider {
    #[must_use]
    pub fn url(&self) -> String {
        match self {
            Self::IpEcho => "https://ipecho.net/ip".to_string(),
            Self::CurlMyIp => "https://curlmyip.net".to_string(),
            Self::MyExternalIp => "https://myexternalip.com/raw".to_string(),
        }
    }
}

pub async fn fetch_ip<'a>(
    protocol: Protocol,
    provider: IpProvider,
) -> Result<(Cow<'a, str>, Cow<'a, str>)> {
    let url = match provider {
        IpProvider::Url(url) => url,
        IpProvider::Embed(embed) => embed.url(),
    };

    match protocol {
        Protocol::V4 => {
            let ip = fetch_ipv4(&url).await?;
            Ok((Cow::Owned(ip), Cow::Borrowed("")))
        }
        Protocol::V6 => {
            let ip = fetch_ipv6(&url).await?;
            Ok((Cow::Borrowed(""), Cow::Owned(ip)))
        }
        Protocol::Both => {
            let (v4_result, v6_result) =
                tokio::join!(fetch_ipv4(&url), fetch_ipv6(&url));
            let v4 = v4_result?;
            let v6 = v6_result?;
            Ok((Cow::Owned(v4), Cow::Owned(v6)))
        }
    }
}

async fn fetch_ipv4(url: &str) -> Result<String> {
    let client = client_builder()
        .local_address("0.0.0.0".parse::<IpAddr>().unwrap())
        .build()?;
    let result = client.get(url).send().await?.text().await?;
    Ok(result.trim().to_string())
}

async fn fetch_ipv6(url: &str) -> Result<String> {
    let client = client_builder()
        .local_address("::".parse::<IpAddr>().unwrap())
        .build()?;
    let result = client.get(url).send().await?.text().await?;
    Ok(result.trim().to_string())
}

fn client_builder() -> ClientBuilder {
    ClientBuilder::new()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .user_agent("Solo")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test() {
        let result = fetch_ip(
            Protocol::Both,
            IpProvider::Embed(EmbedIpProvider::MyExternalIp),
        )
        .await
        .unwrap();
        println!("{result:#?}");
    }
}
