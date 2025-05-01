use serde::{Deserialize, Serialize};

use crate::exec::ipfetcher::{IpProvider, Protocol};

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
