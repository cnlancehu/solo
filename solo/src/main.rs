#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::correctness
)]
use std::{fs, process};

use CliAction::{ManageConfig, RunConfig, ShowHelp, ShowVersion};
use cli::{CliAction, ManageConfigAction, parse, show_conf_help, show_help};
use cnxt::Colorize as _;
use config::{CONFIG_DETECTION_PATH, reader::show_avaliable_configs};
use rust_i18n::set_locale;
use sys_locale::get_locale;

mod cli;
mod config;
mod exec;

rust_i18n::i18n!("locales", fallback = ["en-US"]);

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    cnxt::control::set_virtual_terminal(true);

    if !fs::exists(&*CONFIG_DETECTION_PATH).unwrap_or(false) {
        let _ = fs::create_dir_all(&*CONFIG_DETECTION_PATH);
    }

    let locale = get_locale().unwrap_or_else(|| "en-US".to_string());
    set_locale(&locale);
    println!(
        "{} {}\n",
        "Solo".bright_cyan(),
        format!("v{}", env!("CARGO_PKG_VERSION")).bright_green()
    );

    let action = parse().unwrap_or_else(|_| process::exit(1));

    match action {
        ShowHelp => show_help(),
        RunConfig(config) => {
            exec::run(config).await;
        }
        ManageConfig(action) => match action {
            ManageConfigAction::ShowHelp => show_conf_help(),
            ManageConfigAction::List => show_avaliable_configs(),
            _ => todo!(),
        },
        ShowVersion => todo!(),
    }
}
