use json_spanned_value::Spanned;
use serde::{Deserialize, Serialize};

use crate::exec::ipfetcher::{IpProvider, IpProviderInput, Protocol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MachineType {
    QcloudVpc,
    QcloudLighthouse,
    AliyunEcs,
    AliyunSwas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub servers: Vec<Server>,
    pub schedule: Schedule,
    pub ip_provider: IpProvider,
    pub notifications: Vec<Notification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub name: String,
    pub machine_type: MachineType,
    pub machine_id: String,
    pub region: String,

    pub secret_id: String,
    pub secret_key: String,

    pub protocol: Protocol,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigInput {
    pub name: Spanned<Option<String>>,
    pub servers: Spanned<Option<Vec<ServerInput>>>,
    pub schedule: Option<Schedule>,
    pub ip_provider: Spanned<Option<IpProviderInput>>,
    pub notifications: Spanned<Option<Vec<NotificationInput>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerInput {
    pub name: Spanned<Option<String>>,
    pub machine_type: Spanned<Option<String>>,
    pub machine_id: Spanned<Option<String>>,
    pub region: Spanned<Option<String>>,

    pub secret_id: Spanned<Option<String>>,
    pub secret_key: Spanned<Option<String>>,

    pub protocol: Spanned<Option<String>>,
    pub rules: Spanned<Option<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Schedule {
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

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationInput {
    pub name: Spanned<String>,
    pub trigger: Spanned<String>,
    pub method: Spanned<NotificationMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationTrigger {
    OnSuccess,
    OnSuccessFullyChanged,
    OnFailure,
    /// `OnSuccessFullyChanged` and `OnFailure`
    Both,
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
mod test {
    use std::fs;

    use super::*;

    #[test]
    fn generate_config_example() {
        let config = Config {
            servers: vec![Server {
                machine_type: MachineType::QcloudLighthouse,
                machine_id: "lhins-89onhwdr".to_string(),
                name: "IP262".to_string(),
                region: "ap-guangzhou".to_string(),
                secret_id: "".to_string(),
                secret_key: "".to_string(),
                protocol: Protocol::V4,
                rules: vec![
                    "SSH".to_string(),
                    "BT".to_string(),
                    "ping".to_string(),
                    "frpsadmin".to_string(),
                ],
            }],
            schedule: Schedule::Loop(10),
            ip_provider: IpProvider::Url("https://ipecho.net/ip".to_string()),
            notifications: vec![
                Notification {
                    name: "smtp".to_string(),
                    trigger: NotificationTrigger::OnSuccess,
                    method: NotificationMethod::Smtp {
                        host: "()".to_string(),
                        port: 25,
                        security: SmtpSecurity::TLS,
                        username: "()".to_string(),
                        password: "()".to_string(),
                        from: "()".to_string(),
                        to: "()".to_string(),
                    },
                },
                Notification {
                    name: "qmsg".to_string(),
                    trigger: NotificationTrigger::OnSuccess,
                    method: NotificationMethod::Qmsg {
                        endpoint: None,
                        key: "()".to_string(),
                        msg_type: QmsgConfigMsgType::Group,
                        qq: Some("123456789".to_string()),
                        bot: Some("123456789".to_string()),
                    },
                },
            ],
            name: "IP262".into(),
        };

        let content = serde_json::to_string_pretty(&config).unwrap();
        let _ = fs::write("../conf/241.json", &content).unwrap();
    }
}
