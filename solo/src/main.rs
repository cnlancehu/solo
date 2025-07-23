#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::correctness
)]

use std::{
    env::{self},
    fs, process,
};

use cli::{CliAction, ManageConfigAction};
use cnxt::Colorize as _;
use config::CONFIG_DETECTION_PATH;
use rust_i18n::set_locale;
use sys_locale::get_locale;

use crate::cli::VersionAction;

mod cli;
mod config;
mod exec;

pub mod consts;
pub mod ipfetcher;
pub mod notification;
pub mod report;
pub mod sdk;

rust_i18n::i18n!("locales", fallback = ["en"]);

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    cnxt::control::set_virtual_terminal(true);

    // Create configuration directory if it doesn't exist
    if !fs::exists(&*CONFIG_DETECTION_PATH).unwrap_or(false) {
        let _ = fs::create_dir_all(&*CONFIG_DETECTION_PATH);
    }

    // Set locale
    let locale = env::var("SOLO_LANG")
        .unwrap_or_else(|_| get_locale().unwrap_or_else(|| "en".to_string()));
    set_locale(&locale);

    // Parse command line arguments
    let action = cli::parse().unwrap_or_else(|| process::exit(1));

    // Show version information if needed
    if !matches!(action, CliAction::Version(_)) {
        println!(
            "{} {}\n",
            "Solo".bright_cyan(),
            format!("v{VERSION}").bright_green()
        );
    }

    match action {
        CliAction::ShowHelp => cli::show_help(),
        CliAction::RunConfig(config) => {
            exec::run(config).await;
        }
        CliAction::ManageConfig(action) => match action {
            ManageConfigAction::ShowHelp => cli::config::show_help(),
            ManageConfigAction::List => cli::config::show_avaliable_configs(),
            ManageConfigAction::New => {
                cli::config::new();
            }
        },
        CliAction::Version(action) => match action {
            VersionAction::Show => cli::version::show().await,
            VersionAction::Update => cli::version::update().await,
        },
    }
}
