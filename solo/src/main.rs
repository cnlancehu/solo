#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::correctness
)]

use std::{
    env::{self, current_exe},
    fs,
    path::{Path, PathBuf},
    process,
};

use cli::{
    CliAction::{ManageConfig, RunConfig, ShowHelp, ShowVersion},
    ManageConfigAction, parse, show_conf_help, show_help,
};
use cnxt::Colorize as _;
use config::{CONFIG_DETECTION_PATH, reader::show_avaliable_configs};
use lazy_static::lazy_static;
use rust_i18n::set_locale;
use sys_locale::get_locale;

mod cli;
mod config;
mod exec;

lazy_static! {
    pub static ref EXE_DIR: PathBuf = {
        current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap()
    };
}

rust_i18n::i18n!("locales", fallback = ["en-US"]);

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    cnxt::control::set_virtual_terminal(true);

    if !fs::exists(&*CONFIG_DETECTION_PATH).unwrap_or(false) {
        let _ = fs::create_dir_all(&*CONFIG_DETECTION_PATH);
    }

    // Set locale
    let locale = if let Some(locale) = detect_locale_from_env() {
        locale
    } else {
        get_locale().unwrap_or_else(|| "en-US".to_string())
    };
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

fn detect_locale_from_env() -> Option<String> {
    env::var("SOLO_LANG").ok()
}
