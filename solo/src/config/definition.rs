use serde::{Deserialize, Serialize};

use super::serde::deserialize_untagged_enum_case_insensitive;
use crate::exec::ipfetcher::{IpProvider, Protocol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MachineType {
    QcloudCvm,
    QcloudLighthouse,
    AliyunEcs,
    AliyunSas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub servers: Vec<Server>,
    #[serde(
        default,
        deserialize_with = "deserialize_untagged_enum_case_insensitive"
    )]
    pub schedule: Schedule,
    #[serde(default)]
    pub ip_provider: IpProvider,
    #[serde(default)]
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub name: String,

    #[serde(deserialize_with = "deserialize_untagged_enum_case_insensitive")]
    pub machine_type: MachineType,
    pub machine_id: String,
    pub region: String,

    #[serde(default)]
    pub secret_id: String,
    pub secret_key: String,

    #[serde(deserialize_with = "deserialize_untagged_enum_case_insensitive")]
    pub protocol: Protocol,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Schedule {
    Once,
    Loop(
        usize, // interval_seconds
    ),
}

impl Default for Schedule {
    fn default() -> Self {
        Self::Once
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub name: String,
    #[serde(deserialize_with = "deserialize_untagged_enum_case_insensitive")]
    pub trigger: NotificationTrigger,
    #[serde(deserialize_with = "deserialize_untagged_enum_case_insensitive")]
    pub method: NotificationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationTrigger {
    OnSuccess,
    OnSuccessFullyChanged,
    OnFailure,

    /// `OnSuccessFullyChanged` and `OnFailure`
    Both,
    /// `OnSuccess` and `OnFailure`
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationMethod {
    Smtp {
        host: String,
        port: u16,
        security: SmtpSecurity,
        username: String,
        password: String,
        from: String,
        to: String,
    },
    Qmsg {
        endpoint: Option<String>,
        key: String,
        msg_type: QmsgConfigMsgType,
        show_ipaddr: Option<bool>,

        qq: Option<String>,
        bot: Option<String>,
    },
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmtpSecurity {
    None,
    StartTLS,
    TLS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QmsgConfigMsgType {
    Group,
    Private,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::ipfetcher::EmbedIpProvider;

    #[test]
    fn generate_example() {
        let config = Config {
            name: "example".to_string(),
            servers: vec![],
            schedule: Schedule::Loop(60),
            ip_provider: IpProvider::Embed(EmbedIpProvider::CurlMyIp),
            notifications: vec![],
        };

        let config = toml::to_string(&config).unwrap();
        println!("{}", config);
    }
}
