use std::sync::Arc;

use cnxt::Colorize as _;
use error::{NotificationError, explain_error};

use super::report::{
    ExecutionReport, ExecutionReportIpFetching, ExecutionReportServerStatus,
};
use crate::config::definition::{
    Notification, NotificationMethod, NotificationTrigger,
};

pub mod error;
pub mod qmsg;
pub mod smtp;

#[cfg(target_os = "windows")]
pub mod system;

#[derive(PartialEq, Eq, Clone)]
pub enum Status {
    SuccessButNotChanged,
    SuccessFullyChanged,
    Failed,
}

/// Send multiple notifications based on the execution report.
pub async fn send_notification<'a>(
    notifications: &'a [Notification],
    report: ExecutionReport<'a>,

    max_config_name_length: usize,
    config_num: usize,
) {
    let status = if let ExecutionReportIpFetching::Failed { .. } =
        report.ip_fetching_status
    {
        Status::Failed
    } else {
        // Determine status from server reports
        let has_failure = report.server_status.iter().any(|s| {
            matches!(s.status, ExecutionReportServerStatus::Failed { .. })
        });
        let has_changed = report.server_status.iter().any(|s| {
            matches!(
                s.status,
                ExecutionReportServerStatus::Success {
                    is_ip_changed: true
                }
            )
        });

        if has_failure {
            Status::Failed
        } else if has_changed {
            Status::SuccessFullyChanged
        } else {
            Status::SuccessButNotChanged
        }
    };

    let report = Arc::new(report);
    let status = Arc::new(status);
    let errors: Vec<NotificationError<'a>> = futures::future::join_all(
        notifications
            .iter()
            .filter(|n| should_notify(n, &status))
            .map(|n| {
                let report = report.clone();
                let status = status.clone();
                async move { send_single_notification(n, report, status).await }
            }),
    )
    .await
    .into_iter()
    .flatten()
    .collect();

    if !errors.is_empty() {
        let error_message = explain_error(errors);
        for msg in error_message {
            if config_num == 1 {
                println!("{msg}");
            } else {
                println!(
                    "{:<max_config_name_length$} | {}",
                    report.config_name,
                    msg.bright_red()
                );
            }
        }
    }
}

fn should_notify(notification: &Notification, status: &Status) -> bool {
    match notification.trigger {
        NotificationTrigger::OnSuccess => {
            *status == Status::SuccessFullyChanged
                || *status == Status::SuccessButNotChanged
        }
        NotificationTrigger::OnFailure => *status == Status::Failed,
        NotificationTrigger::OnSuccessFullyChanged => {
            *status == Status::SuccessFullyChanged
        }
        NotificationTrigger::Both => {
            *status == Status::SuccessFullyChanged || *status == Status::Failed
        }
        NotificationTrigger::Always => true,
    }
}

async fn send_single_notification<'a>(
    notification: &'a Notification,
    report: Arc<ExecutionReport<'a>>,
    status: Arc<Status>,
) -> Option<NotificationError<'a>> {
    match notification.method {
        NotificationMethod::Smtp { .. } => {
            smtp::send(notification, &report, &status)
        }
        NotificationMethod::Qmsg { .. } => {
            qmsg::send(notification, &report, &status).await
        }
        NotificationMethod::System => {
            #[cfg(target_os = "windows")]
            {
                system::send(notification, &report, &status).await
            }
            #[cfg(not(target_os = "windows"))]
            {
                None
            }
        }
    }
}
