use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time,
};

use futures::TryFutureExt as _;
use rand::Rng;
use rust_i18n::t;
use tauri_winrt_notification::{Duration, IconCrop, Progress, Sound, Toast};
use tokio::{fs, time::sleep};

use super::{Status, error::NotificationError};
use crate::{
    config::definition::Notification,
    consts::EXE_DIR,
    report::{ExecutionReport, show_full_report},
};

pub async fn send<'a>(
    notification: &'a Notification,
    report: &ExecutionReport<'a>,
    status: &Arc<Status>,
) -> Option<NotificationError<'a>> {
    send_child(report, status)
        .map_err(|e| NotificationError {
            name: Cow::Borrowed(&notification.name),
            error: e,
        })
        .await
        .err()
}

async fn send_child<'a>(
    report: &ExecutionReport<'a>,
    status: &Status,
) -> Result<(), Cow<'static, str>> {
    let icon_path = EXE_DIR.join("soloicon.ico");
    let is_success = match status {
        Status::Failed => false,
        Status::SuccessButNotChanged | Status::SuccessFullyChanged => true,
    };
    if is_success {
        let _ = fs::write(
            &icon_path,
            include_bytes!("../../../assets/circle-check.ico"),
        )
        .await;
    } else {
        let _ = fs::write(
            &icon_path,
            include_bytes!("../../../assets/circle-xmark.ico"),
        )
        .await;
    }
    let title = if is_success {
        t!("Solo: Execution successful")
    } else {
        t!("Solo: Execution failed")
    };
    let text = match status {
        Status::Failed => t!("Click to view error details"),
        Status::SuccessButNotChanged => t!("IP unchanged"),
        Status::SuccessFullyChanged => t!("IP changed successfully"),
    };

    let mut toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(&title)
        .text1(&text)
        .icon(&icon_path, IconCrop::Square, "")
        .sound(Some(Sound::Reminder));
    if is_success {
        toast = toast.duration(Duration::Short);

        toast.show().map_err(|e| -> Cow<'static, str> {
            Cow::Owned(
                t!(
                    "Unable to send system notification | %{error}",
                    error = e.to_string()
                )
                .to_string(),
            )
        })?;
    } else {
        let mut progress = Progress {
            tag: generate_random_id(),
            title: t!("Please act within %{time} seconds", time = 30)
                .to_string(),
            status: t!("Countdown").to_string(),
            value: 100.0,
            value_string: "30s".to_string(),
        };

        let activated = Arc::new(AtomicBool::new(false));
        let activated_check = activated.clone();

        let report = show_full_report(report, false, true).join("\n");
        toast = toast
            .duration(Duration::Long)
            .add_button(&t!("View error details"), "1")
            .add_button(&t!("Cancel"), "2")
            .on_activated({
                move |action| {
                    // Process the action, or execute the same logic if action is None
                    if action.is_some() && action.as_deref() == Some("1")
                        || action.is_none()
                    {
                        let report_path = EXE_DIR.join("soloreport.txt");
                        let _ = std::fs::write(&report_path, &report);
                        let _ = opener::open(report_path);
                    }
                    activated_check.store(true, Ordering::SeqCst);
                    Ok(())
                }
            })
            .progress(&progress);

        toast.show().map_err(|e| -> Cow<'static, str> {
            Cow::Owned(
                t!(
                    "Unable to send system notification | %{error}",
                    error = e.to_string()
                )
                .to_string(),
            )
        })?;

        for i in (0..30).rev() {
            if activated.load(Ordering::SeqCst) {
                break;
            }
            sleep(time::Duration::from_secs(1)).await;
            if i == 0 {
                progress.value = 0.0;
                progress.value_string = String::new();
                progress.title = t!("Timed out", time = i).to_string();
                progress.status =
                    t!("Please manually run Solo to view error details")
                        .to_string();
            } else {
                progress.value = i as f32 / 30.0;
                progress.value_string = format!("{i}s");
                progress.title =
                    t!("Please act within %{time} seconds", time = i)
                        .to_string();
            }
            toast.set_progress(&progress).map_err(
                |e| -> Cow<'static, str> {
                    Cow::Owned(
                        t!(
                            "Unable to send system notification | %{error}",
                            error = e.to_string()
                        )
                        .to_string(),
                    )
                },
            )?;
        }
    }

    Ok(())
}

fn generate_random_id() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..6)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}
