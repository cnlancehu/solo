use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
    thread,
};

use reqwest::Client;
use tokio::sync::mpsc::Sender;

use super::{
    ThreadStep,
    report::{ExecutionReportServer, ExecutionReportServerStatus},
};
use crate::config::definition::{MachineType, Server};

mod aliyun;
mod qcloud;

pub async fn execute_server_task<'a>(
    tx: Sender<ThreadStep>,

    client: &Client,
    server: Server,

    ipv4: Cow<'static, str>,
    ipv6: Cow<'static, str>,
) -> ExecutionReportServer<'a> {
    let (txx, rxx) = std::sync::mpsc::channel::<Cow<'static, str>>();
    let server_name: Cow<'static, str> = Cow::Owned(server.name.clone());
    let server_name_clone = server_name.clone();
    let step_msg = Arc::new(Mutex::new(Cow::Borrowed("")));
    let step_msg_clone = step_msg.clone();
    let tx_clone = tx.clone();

    thread::spawn(move || {
        while let Ok(message) = rxx.recv() {
            (*step_msg_clone.lock().unwrap()).clone_from(&message);
            let _ = tx_clone.blocking_send(ThreadStep {
                name: Some(server_name_clone.clone()),
                msg: message.clone(),
            });
        }
    });

    let result = match server.machine_type {
        MachineType::QcloudVpc => {
            qcloud::vpc(txx, client, server, ipv4, ipv6).await
        }
        MachineType::QcloudLighthouse => {
            qcloud::lighthouse(txx, client, server, ipv4).await
        }
        MachineType::AliyunEcs => {
            aliyun::ecs(txx, client, server, ipv4, ipv6).await
        }
        MachineType::AliyunSwas => {
            aliyun::swas(txx, client, server, ipv4).await
        }
    };
    match result {
        Ok(r) => r,
        Err(e) => ExecutionReportServer {
            status: ExecutionReportServerStatus::Failed {
                error: e.into(),
                when: step_msg.lock().unwrap().clone(),
            },
            name: server_name,
        },
    }
}
