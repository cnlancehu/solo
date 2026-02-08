use serde::{Deserialize, Serialize};

use crate::ipfetcher::{IpProvider, Protocol};

pub const MACHINE_TYPES_WITH_OPTIONAL_SECRET_ID: &[MachineType] =
    &[MachineType::RainyunRcs];

pub const MACHINE_TYPES_WITH_OPTIONAL_REGION_ID: &[MachineType] =
    &[MachineType::RainyunRcs];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MachineType {
    QcloudCvm,
    QcloudLighthouse,
    AliyunEcs,
    AliyunSas,
    RainyunRcs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub schedule: Schedule,
    #[serde(default)]
    pub ip_provider: IpProvider,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notifications: Vec<Notification>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub name: String,

    pub machine_type: MachineType,
    pub machine_id: String,

    #[serde(default)]
    pub region: String,

    #[serde(default)]
    pub secret_id: String,
    pub secret_key: String,

    pub protocol: Protocol,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Schedule {
    #[default]
    Once,
    Loop(
        usize, // interval_seconds
    ),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub name: String,
    pub trigger: NotificationTrigger,
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
    use crate::ipfetcher::EmbedIpProvider;

    #[test]
    fn generate_example() {
        let config = Config {
            name: "example".to_string(),
            servers: vec![],
            schedule: Schedule::Loop(60),
            ip_provider: IpProvider::Embed(EmbedIpProvider::CurlMyIp),
            notifications: vec![Notification {
                name: "qmsg".to_string(),
                trigger: NotificationTrigger::OnSuccessFullyChanged,
                method: NotificationMethod::Qmsg {
                    endpoint: Some("endpoint".to_string()),
                    key: "key".to_string(),
                    msg_type: QmsgConfigMsgType::Group,
                    show_ipaddr: Some(true),
                    qq: Some("qq".to_string()),
                    bot: Some("bot".to_string()),
                },
            }],
            no_proxy: false,
        };

        let config = toml::to_string(&config).unwrap();
        println!("{config}");
    }
}
