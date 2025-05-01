use error::NotificationError;

use super::report::{
    ExecutionReport, ExecutionReportIpFetching, ExecutionReportServerStatus,
};
use crate::config::definition::{
    Notification, NotificationMethod, NotificationTrigger,
};

pub mod error;
pub mod qmsg;
pub mod smtp;

#[derive(PartialEq, Eq, Clone)]
pub enum Status {
    SuccessButNotChanged,
    SuccessFullyChanged,
    Failed,
}

pub async fn send_notification<'a>(
    notifications: &'a [Notification],
    report: &'a ExecutionReport<'static>,
) -> Vec<NotificationError<'a>> {
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

    // 使用迭代器和过滤器直接收集结果，减少中间集合
    futures::future::join_all(
        notifications
            .iter()
            .filter(|n| should_notify(n, &status))
            .map(|n| async {
                send_single_notification(
                    n,
                    report,
                    status.clone(),
                )
                .await
            }),
    )
    .await
    .into_iter()
    .flatten()
    .collect()
}

// 提取过滤逻辑为单独函数，提高可读性
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
    }
}
async fn send_single_notification<'a>(
    notification: &'a Notification,
    report: &'a ExecutionReport<'a>,
    status: Status,
) -> Option<NotificationError<'a>> {
    match notification.method {
        NotificationMethod::Smtp { .. } => {
            smtp::send(notification, report, status)
        }
        NotificationMethod::Qmsg { .. } => {
            qmsg::send(notification, report, status).await
        }
        NotificationMethod::System => todo!(),
    }
}
