use std::{borrow::Cow, sync::mpsc::Sender};

use anyhow::Result;
use reqwest::Client;
use rust_i18n::t;

use crate::{
    config::definition::Server,
    report::{ExecutionReportServer, ExecutionReportServerStatus},
};

pub async fn rcs<'a>(
    tx: Sender<Cow<'a, str>>,

    client: &Client,
    server: Server,

    ipv4: Cow<'a, str>,
) -> Result<ExecutionReportServer<'a>> {
    use solo_lib::sdk::rainyun::rcs::{
        compare_rules, list_rules, modify_rules,
    };
    let server_name = Cow::<'a, str>::Owned(server.name.clone());

    let send = move |msg: Cow<'a, str>| {
        let _ = tx.send(msg);
    };

    let instance_id = server.machine_id;
    let token = server.secret_key;
    send(t!("获取防火墙规则"));
    let response = list_rules(client, &instance_id, &token).await?;
    let records = response.data.records;
    let (records, require_update) =
        compare_rules(&records, &ipv4, &server.rules);
    if require_update {
        send(t!("修改防火墙规则"));
        modify_rules(client, &instance_id, &token, &records).await?;

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
