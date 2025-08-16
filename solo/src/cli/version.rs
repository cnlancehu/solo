use std::{
    borrow::Cow,
    fs::{self, OpenOptions},
    io::{Write as _, stdout},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use cnxt::Colorize as _;
use crossterm::{
    cursor::MoveToColumn,
    queue,
    terminal::{Clear, ClearType},
};
use futures_util::StreamExt as _;
use rust_i18n::t;
use serde::Deserialize;
use solo_lib::client;
use tokio::{task, time::sleep};

use crate::{
    VERSION,
    cli::{
        CliAction, HelpInfo, VersionAction,
        util::{
            HELP_ARGS, HelpSubcommand, build_help_subcommands, print_error_info,
        },
    },
    consts::{EXE_DIR, EXE_NAME},
};

pub const BUILD_TIME: &str = env!("SOLO_BUILD_TIME");
pub const TARGET: &str = env!("SOLO_TARGET");
pub const TARGET_OS_DISPLAY: &str = env!("SOLO_TARGET_OS_DISPLAY");
pub const TARGET_ARCH_DISPLAY: &str = env!("SOLO_TARGET_ARCH_DISPLAY");

#[derive(Debug, Deserialize)]
struct CheckUpdateResponseArtifact {
    file_name: String,
    download_url: String,
}

#[derive(Debug, Deserialize)]
struct CheckUpdateResponse {
    /// Status code indicating version comparison result:
    /// * 0: Current version is the latest
    /// * 1: New version available for update
    /// * -1: Current version is a preview/development version
    status: i32,
    latest_version: Option<String>,
    artifact: Option<Vec<CheckUpdateResponseArtifact>>,
}

pub fn handle_version_command(
    args: &[String],
    args_quantity: usize,
) -> Option<CliAction> {
    match args_quantity {
        2 => Some(CliAction::Version(VersionAction::Show)),
        3 => match args[2].as_str() {
            arg if HELP_ARGS.contains(&arg) => {
                Some(CliAction::ShowHelp(HelpInfo::Version))
            }
            "show" => Some(CliAction::Version(VersionAction::Show)),
            "update" => Some(CliAction::Version(VersionAction::Update)),
            _ => {
                print_error_info(
                    &[2],
                    &t!("Unknown version command"),
                    Some(&t!(
                        "Type %{cmd} for help",
                        cmd = format!("`{} version help`", *EXE_NAME)
                    )),
                );
                None
            }
        },
        _ => {
            print_error_info(
                &[3],
                &t!("This command does not support more parameters"),
                Some(&t!("Remove extra parameters and try again")),
            );
            None
        }
    }
}

pub async fn show() {
    println!(
        "{} {} {} {} {}",
        "Solo".bright_cyan(),
        format!("v{VERSION}").bright_green(),
        "-".bright_white(),
        TARGET_OS_DISPLAY.bright_magenta(),
        TARGET_ARCH_DISPLAY.bright_yellow()
    );
    println!(
        "{} {}",
        t!("Build at").bright_white(),
        BUILD_TIME.bright_blue()
    );
    println!();

    let mut stdout = stdout();

    print!("{}", t!("Checking for updates...").bright_yellow());
    let _ = stdout.flush();
    let response = check_update().await;
    let _ = queue!(stdout, Clear(ClearType::CurrentLine), MoveToColumn(0));

    match response {
        Ok(response) => match response.status {
            0 => {
                println!(
                    "{}",
                    t!("You are using the latest version.").bright_green()
                );
            }
            1 => {
                println!("{}", t!("New version available:").bright_yellow());
                println!(
                    "   {} => {}",
                    VERSION.bright_red(),
                    response
                        .latest_version
                        .unwrap_or_else(|| "Unknown".to_string())
                        .bright_green()
                );
                println!(
                    "{}",
                    t!(
                        "Run %{cmd} to update",
                        cmd = format!("`{} version update`", *EXE_NAME)
                    )
                    .bright_cyan()
                );
            }
            -1 => {
                println!(
                    "{}",
                    t!("You are using a preview/development version.")
                        .bright_red()
                );
                println!(
                    "{}",
                    t!("Can be upgraded to the latest stable version:")
                        .bright_yellow()
                );
                println!(
                    "   {} => {}",
                    VERSION.bright_red(),
                    response
                        .latest_version
                        .unwrap_or_else(|| "Unknown".to_string())
                        .bright_green()
                );
                println!(
                    "{}",
                    t!(
                        "Run %{cmd} to update",
                        cmd = format!("`{} version update`", *EXE_NAME)
                    )
                    .bright_cyan()
                );
            }
            _ => (),
        },
        Err(err) => {
            eprintln!("{}", err.bright_red());
        }
    }
}

pub async fn update() {
    let mut stdout = stdout();

    print!("{}", t!("Checking for updates...").bright_yellow());
    let _ = stdout.flush();
    let response = check_update().await;
    let _ = queue!(stdout, Clear(ClearType::CurrentLine), MoveToColumn(0));

    let response = match response {
        Ok(r) => r,
        Err(err) => {
            println!("{}", err.bright_red());
            return;
        }
    };

    match response.status {
        0 => {
            println!(
                "{}",
                t!("You are using the latest version.").bright_green()
            );
        }
        1 | -1 => {
            println!("{}", t!("Performing update").bright_yellow());
            println!(
                "{} => {}",
                VERSION.bright_red(),
                response
                    .latest_version
                    .unwrap_or_else(|| "Unknown".to_string())
                    .bright_green()
            );
            println!();

            let client = client::new();
            let artifacts = response.artifact.unwrap_or_default();
            let artifacts_len = artifacts.len();

            for (i, artifact) in artifacts.into_iter().enumerate() {
                let idx = i + 1;
                let response = client.get(&artifact.download_url).send().await;
                if let Ok(response) = response {
                    let total_size = response.content_length().unwrap_or(0);
                    let mut stream = response.bytes_stream();
                    let mut content = Vec::new();

                    let downloaded = Arc::new(AtomicU64::new(0));

                    let progress = downloaded.clone();
                    let handle = task::spawn(async move {
                        let mut task_stdout = std::io::stdout();
                        loop {
                            let done = progress.load(Ordering::Relaxed);
                            let percent = if total_size > 0 {
                                (done as f64 / total_size as f64 * 100.0)
                                    .round()
                                    as u64
                            } else {
                                0u64
                            };

                            let _ = queue!(
                                task_stdout,
                                Clear(ClearType::CurrentLine),
                                MoveToColumn(0)
                            );
                            print!(
                                "{} | {}% | {}/{}",
                                t!("Downloading").bright_yellow(),
                                format!("{percent:>3}").bright_green(),
                                idx.to_string().bright_cyan(),
                                artifacts_len.to_string().bright_cyan()
                            );
                            let _ = task_stdout.flush();

                            if total_size > 0 && done >= total_size {
                                break;
                            }
                            sleep(Duration::from_millis(50)).await;
                        }
                    });

                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(data) => {
                                let len = data.len() as u64;
                                downloaded.fetch_add(len, Ordering::Relaxed);
                                content.extend(data);
                            }
                            Err(e) => {
                                eprintln!(
                                    "{}",
                                    t!("Download error: %{error}", error = e)
                                        .bright_red()
                                );
                                return;
                            }
                        }
                    }

                    handle.await.unwrap();

                    if let Err(e) = force_write(
                        &content,
                        &EXE_DIR.join(&artifact.file_name),
                    ) {
                        eprintln!(
                            "{}",
                            t!("Failed to write file: %{error}", error = e)
                                .bright_red()
                        );
                        return;
                    }
                } else {
                    eprintln!(
                        "{}",
                        t!("Failed to download artifact").bright_red()
                    );
                }
            }

            let _ =
                queue!(stdout, Clear(ClearType::CurrentLine), MoveToColumn(0));
            println!("{}", t!("Complete").bright_green());
        }
        _ => (),
    }
}

async fn check_update() -> Result<CheckUpdateResponse, Cow<'static, str>> {
    let url =
        format!("https://pkg.lance.fun/check_update?solo+{VERSION}+{TARGET}");
    let response = client::new()
        .get(&url)
        .send()
        .await
        .map_err(|_| t!("Network error | Unable to connect to server"))?;
    if response.status().is_success() {
        let response =
            response.json::<CheckUpdateResponse>().await.map_err(|_| {
                t!("Network error | Unable to process response content")
            })?;
        Ok(response)
    } else {
        Err(t!("Network error | Unable to connect to server"))
    }
}

fn force_write(content: &[u8], to: &PathBuf) -> Result<(), std::io::Error> {
    // First try normal write
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to)
        && file.write_all(content).is_ok()
        && file.flush().is_ok()
    {
        return Ok(());
    }

    // If normal write fails, use atomic replacement strategy
    let temp_path = to.with_extension("tmp");

    // Write to temporary file first
    {
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;
        temp_file.write_all(content)?;
        temp_file.flush()?;
    }

    if cfg!(windows) {
        // On Windows, if the file is in use (like current exe), rename it first
        let backup_path = to.with_extension("old");

        // Try to rename original to backup
        let _ = fs::rename(to, &backup_path);

        // Move temp file to target location
        fs::rename(&temp_path, to)?;

        // Clean up backup file
        let _ = fs::remove_file(&backup_path);
    } else {
        // On Unix-like systems, atomic replacement
        fs::rename(&temp_path, to)?;
    }

    Ok(())
}

// Show the help message for the `version` command
pub fn show_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("Usage:").bright_green(),
        EXE_NAME.bright_cyan(),
        "version".bright_yellow(),
        t!("[command]").bright_blue()
    ));
    help.push(format!("{}", t!("Available commands:").bright_green()));
    let subcommands: Vec<HelpSubcommand> = vec![
        HelpSubcommand {
            name: "show",
            additional_arg: None,
            description: t!("Show version information"),
        },
        HelpSubcommand {
            name: "update",
            additional_arg: None,
            description: t!("Perform version update"),
        },
    ];
    help.extend(build_help_subcommands(subcommands));

    for line in help {
        println!("{line}");
    }
}
