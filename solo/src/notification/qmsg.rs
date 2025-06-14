use std::{borrow::Cow, sync::Arc};

use anyhow::Result;
use futures::TryFutureExt as _;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use solo_lib::client;

use super::{Status, error::NotificationError};
use crate::{
    config::definition::{Notification, NotificationMethod, QmsgConfigMsgType},
    report::{ExecutionReport, show_full_report},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QmsgRequest {
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    qq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bot: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct QmsgResponse {
    success: bool,
    reason: String,
    code: i32,
}

pub async fn send<'a>(
    notification: &'a Notification,
    report: &ExecutionReport<'a>,
    status: &Arc<Status>,
) -> Option<NotificationError<'a>> {
    send_child(notification, report, status)
        .map_err(|e| NotificationError {
            name: Cow::Borrowed(&notification.name),
            error: e,
        })
        .await
        .err()
}

async fn send_child(
    notification: &Notification,
    report: &ExecutionReport<'_>,
    status: &Status,
) -> Result<(), Cow<'static, str>> {
    let NotificationMethod::Qmsg {
        endpoint,
        key,
        show_ipaddr,
        msg_type,
        qq,
        bot,
    } = &notification.method
    else {
        unreachable!()
    };
    let report = show_full_report(report, false, show_ipaddr.unwrap_or(false));

    let endpoint = endpoint.as_ref().map_or("https://qmsg.zendee.cn", |v| v);
    let url = match msg_type {
        QmsgConfigMsgType::Group => format!("{endpoint}/jgroup/{key}"),
        QmsgConfigMsgType::Private => format!("{endpoint}/jsend/{key}"),
    };

    let handle_error = |e: reqwest::Error| -> Cow<'static, str> {
        t!("无法连接 | %{error}", error = e.to_string())
    };

    let subject = match status {
        Status::Failed => t!("Solo: 运行失败"),
        Status::SuccessButNotChanged => t!("Solo: 运行成功 | IP 相同"),
        Status::SuccessFullyChanged => t!("Solo: 运行成功 | IP 更改成功"),
    };

    let request = QmsgRequest {
        msg: format!("{}\n\n{}", subject, report.join("\n")),
        qq: qq.clone(),
        bot: bot.clone(),
    };

    let response = client::new()
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(handle_error)?;
    let response = response.text().await.map_err(handle_error)?;
    let response: QmsgResponse =
        serde_json::from_str(&response).map_err(|e| {
            t!("解析返回数据错误 | %{error}", error = e.to_string())
        })?;
    if !response.success {
        return Err(t!("发送失败 | %{reason}", reason = response.reason));
    }

    Ok(())
}
