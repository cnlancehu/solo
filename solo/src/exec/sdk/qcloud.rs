use std::{borrow::Cow, sync::mpsc::Sender};

use anyhow::Result;
use reqwest::Client;
use rust_i18n::t;
use solo_lib::sdk::qcloud::Secret;

use crate::{
    config::definition::Server,
    exec::report::{ExecutionReportServer, ExecutionReportServerStatus},
};

pub async fn vpc<'a>(
    tx: Sender<Cow<'a, str>>,

    client: &Client,
    server: Server,

    ipv4: Cow<'a, str>,
    ipv6: Cow<'a, str>,
) -> Result<ExecutionReportServer<'a>> {
    use solo_lib::sdk::qcloud::vpc::{
        SecurityGroup, compare_rules, list_rules, modify_rules,
    };

    let server_name = Cow::<'a, str>::Owned(server.name.clone());

    let send = move |msg: Cow<'a, str>| {
        let _ = tx.send(msg);
    };

    let security_group = SecurityGroup {
        id: server.machine_id,
        region: server.region,
    };
    let secret = Secret {
        secret_id: server.secret_id,
        secret_key: server.secret_key,
    };
    send(t!("获取防火墙规则"));
    let response = list_rules(client, &security_group, &secret).await?;
    let security_group_policy_set =
        response.response.data.security_group_policy_set;
    let (security_group_policy_set, require_update) =
        compare_rules(&security_group_policy_set, &ipv4, &ipv6, &server.rules);
    if require_update {
        send(t!("修改防火墙规则"));
        modify_rules(
            client,
            &security_group,
            &secret,
            &security_group_policy_set,
        )
        .await?;

        Ok(ExecutionReportServer {
            name: server_name,
            status: ExecutionReportServerStatus::Success {
                is_ip_changed: true,
            },
        })
    } else {
        Ok(ExecutionReportServer {
            name: server_name,
            status: ExecutionReportServerStatus::Success {
                is_ip_changed: false,
            },
        })
    }
}

pub async fn lighthouse<'a>(
    tx: Sender<Cow<'a, str>>,

    client: &Client,
    server: Server,

    ipv4: Cow<'a, str>,
) -> Result<ExecutionReportServer<'a>> {
    use solo_lib::sdk::qcloud::lighthouse::{
        Instance, compare_rules, list_rules, modify_rules,
    };

    let server_name = Cow::<'a, str>::Owned(server.name.clone());

    let send = move |msg: Cow<'a, str>| {
        let _ = tx.send(msg);
    };

    let instance = Instance {
        id: server.machine_id,
        region: server.region,
    };
    let secret = Secret {
        secret_id: server.secret_id,
        secret_key: server.secret_key,
    };
    send(t!("获取防火墙规则"));
    let response = list_rules(client, &instance, &secret).await?;
    let firewall_rules = &response.response.data.firewall_rule_set;
    let (firewall_rules, require_update) =
        compare_rules(firewall_rules, &ipv4, &server.rules);
    if require_update {
        send(t!("修改防火墙规则"));
        modify_rules(client, &instance, &secret, &firewall_rules).await?;

        Ok(ExecutionReportServer {
            name: server_name,
            status: ExecutionReportServerStatus::Success {
                is_ip_changed: true,
            },
        })
    } else {
        Ok(ExecutionReportServer {
            name: server_name,
            status: ExecutionReportServerStatus::Success {
                is_ip_changed: false,
            },
        })
    }
}
