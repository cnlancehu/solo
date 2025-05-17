use std::{borrow::Cow, sync::Arc};

use anyhow::Result;
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::{self, authentication::Credentials},
};
use rust_i18n::t;

use super::{Status, error::NotificationError};
use crate::{
    config::definition::{Notification, NotificationMethod, SmtpSecurity},
    exec::report::{ExecutionReport, show_full_report},
};

pub fn send<'a>(
    notification: &'a Notification,
    report: &ExecutionReport<'a>,
    status: &Arc<Status>,
) -> Option<NotificationError<'a>> {
    let report = show_full_report(report, false, true);

    send_child(notification, &report, &status)
        .map_err(|e| NotificationError {
            name: Cow::Borrowed(&notification.name),
            error: e,
        })
        .err()
}

fn send_child(
    notification: &Notification,
    report: &[String],
    status: &Status,
) -> Result<(), Cow<'static, str>> {
    let NotificationMethod::Smtp {
        host,
        port,
        security,
        username,
        password,
        from,
        to,
    } = &notification.method
    else {
        unreachable!()
    };

    let handle_error = |e: smtp::Error| -> Cow<'static, str> {
        t!("无法连接至邮箱 | %{error}", error = e.to_string())
    };

    let subject = if status == &Status::Failed {
        t!("Solo: 运行失败")
    } else {
        t!("Solo: 运行成功")
    };

    let email = Message::builder()
        .subject(subject)
        .user_agent("Solo".to_string())
        .header(ContentType::TEXT_PLAIN)
        .date_now()
        .from(from.parse().map_err(|_| t!("发信人地址填写错误"))?)
        .to(to.parse().map_err(|_| t!("收信人地址填写错误"))?)
        .body(report.join("\n"))
        .map_err(|e| {
            t!("邮件构建填写错误 | %{error}", error = e.to_string())
        })?;

    let creds = Credentials::new(username.clone(), password.clone());

    let mailer = match security {
        SmtpSecurity::None => SmtpTransport::builder_dangerous(host)
            .port(*port)
            .credentials(creds)
            .build(),
        SmtpSecurity::StartTLS => SmtpTransport::starttls_relay(host)
            .map_err(handle_error)?
            .port(*port)
            .credentials(creds)
            .build(),
        SmtpSecurity::TLS => SmtpTransport::relay(host)
            .map_err(handle_error)?
            .port(*port)
            .credentials(creds)
            .build(),
    };

    mailer
        .send(&email)
        .map_err(|e| t!("邮件发送失败 | %{error}", error = e.to_string()))?;

    Ok(())
}
