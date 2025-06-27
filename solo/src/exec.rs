use std::{borrow::Cow, sync::Arc, time::Duration};

use chrono::Local;
use cnxt::Colorize;
use futures::stream::{FuturesUnordered, StreamExt};
use hashbrown::HashMap;
use ipfetcher::{Protocol, fetch_ip};
use notification::send_notification;
use report::{ExecutionReport, ExecutionReportIpFetching, show_brief_report};
use rust_i18n::t;
use sdk::execute_server_task;
use solo_lib::client;
use tokio::{
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};
use unicode_width::UnicodeWidthStr as _;

use crate::{
    config::{
        definition::{Config, Schedule},
        reader::process_config,
    },
    ipfetcher, notification, report, sdk,
};

#[derive(Debug, Clone)]
pub struct ThreadStep {
    pub(crate) name: Option<Cow<'static, str>>,
    pub(crate) msg: Cow<'static, str>,
}

pub async fn run(config_args: Vec<String>) {
    let config: Vec<Config> = process_config(config_args).unwrap();
    let config_num = config.len();
    if config_num > 1 {
        println!(
            "{}",
            t!("Running multiple configurations, Solo will not output detailed runtime information").bright_yellow()
        );
    }
    let (tx, mut rx) = mpsc::channel(100);

    let max_config_name_length = config
        .iter()
        .map(|config| config.name.width())
        .max()
        .unwrap_or(0);

    let mut id_config_name = HashMap::new();
    let mut id_config_schedule = HashMap::new();
    let mut id_config_notifications = HashMap::new();
    let mut futures = FuturesUnordered::new();

    for (id, config) in config.iter().enumerate() {
        let config_name = config.name.clone();
        id_config_name.insert(id, config_name);
        id_config_schedule.insert(id, config.schedule.clone());
        id_config_notifications.insert(id, config.notifications.clone());
        let handle = execute_task(id, tx.clone(), config.clone(), 0);
        futures.push(handle);
    }

    let mut active_tasks = config.len();
    loop {
        tokio::select! {
            Some(step) = rx.recv() => {
                if config_num == 1 {
                    if let Some(name) = step.name {
                        println!(
                            "[{}] {}", name.bright_magenta(), step.msg.bright_cyan()
                        );
                    } else {
                        println!(
                            "{}",
                            step.msg.bright_cyan()
                        );
                    }
                }
            }
            Some(Ok(report)) = futures.next() => {
                let content_report = show_brief_report(&report, true);
                let content_report = if config_num == 1 {
                    content_report.join("\n")
                } else {
                    content_report
                        .iter()
                        .map(|line| format!("{:<max_config_name_length$} | {}", "", line))
                        .collect::<Vec<_>>().join("\n")
                };
                println!("{content_report}");

                match id_config_schedule.get(&report.id) {
                    Some(Schedule::Once) => {
                        if config_num == 1 {
                            println!(
                                "{}",
                                t!("Execution completed").bright_green()
                            );
                        } else {
                            println!(
                                "{:<max_config_name_length$} | {}",
                                id_config_name.get(&report.id).map_or("", |v| v).bright_yellow(),
                                t!("Execution completed").bright_green()
                            );
                        }
                        active_tasks -= 1;

                        send_notification(
                            id_config_notifications.get(&report.id).unwrap(),
                            report,

                            1,
                            0,
                        ).await;
                    },
                    Some(Schedule::Loop(interval)) => {
                        if config_num == 1 {
                            println!(
                                "{}",
                                t!("Waiting for the next execution").bright_yellow()
                            );
                        } else {
                            println!(
                                "{:<max_config_name_length$} | {}",
                                id_config_name.get(&report.id).map_or("", |v| v).bright_yellow(),
                                t!("Waiting for the next execution").bright_yellow()
                            );
                        }

                        let handle = execute_task(report.id, tx.clone(), config[report.id].clone(), *interval);
                        futures.push(handle);

                        if let Some(notifications) = id_config_notifications.get(&report.id).cloned() {
                            tokio::task::spawn_blocking(move || {
                                tokio::runtime::Handle::current().block_on(async {
                                    send_notification(
                                        &notifications,
                                        report,
                                        max_config_name_length,
                                        config_num,
                                    ).await;
                                });
                            });
                        }
                    }
                    None => (),
                }

                if active_tasks == 0 {
                    break;
                }
            }
            else => {
                break;
            }
        }
    }
}

fn execute_task<'a>(
    id: usize,
    tx: Sender<ThreadStep>,
    config: Config,
    sleep_interval: usize,
) -> JoinHandle<ExecutionReport<'a>> {
    let tx_clone = tx.clone();
    let send = move |name: Option<&str>, msg: Cow<'_, str>| {
        let _ = tx_clone.try_send(ThreadStep {
            name: name.map(|name| Cow::Owned(name.to_string())),
            msg: Cow::Owned(msg.into_owned()),
        });
    };

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(sleep_interval as u64)).await;

        send(None, t!("Fetching IP address"));
        let protocol = if config
            .servers
            .iter()
            .any(|server| server.protocol == Protocol::Both)
            || (config
                .servers
                .iter()
                .any(|server| server.protocol == Protocol::V4)
                && config
                    .servers
                    .iter()
                    .any(|server| server.protocol == Protocol::V6))
        {
            Protocol::Both
        } else {
            config.servers[0].protocol
        };

        match fetch_ip(protocol, config.ip_provider).await {
            Ok((ipv4, ipv6)) => {
                let ipfetching_result = ExecutionReportIpFetching::Success {
                    ipv4: ipv4.clone(),
                    ipv6: ipv6.clone(),
                };

                let mut builder = client::new_builder();
                if config.no_proxy.unwrap_or(false) {
                    builder = builder.no_proxy();
                }
                let client = builder.build().unwrap();

                let mut server_result = Vec::new();
                for server in config.servers {
                    let result = execute_server_task(
                        tx.clone(),
                        &client,
                        server.clone(),
                        ipv4.clone(),
                        ipv6.clone(),
                    )
                    .await;
                    server_result.push(result);
                }
                let finished_timestamp = Local::now().timestamp();
                ExecutionReport {
                    id,
                    config_name: Cow::Owned(config.name.clone()),
                    finished_timestamp,
                    ip_fetching_status: ipfetching_result,
                    server_status: server_result,
                }
            }
            Err(e) => {
                let ipfetching_result =
                    ExecutionReportIpFetching::Failed { error: Arc::new(e) };
                let finished_timestamp = Local::now().timestamp();
                ExecutionReport {
                    id,
                    config_name: Cow::Owned(config.name.clone()),
                    finished_timestamp,
                    ip_fetching_status: ipfetching_result,
                    server_status: vec![],
                }
            }
        }
    })
}
