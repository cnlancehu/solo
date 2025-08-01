pub mod conf;
mod conf_new;
pub mod go;
pub mod util;
pub mod version;

use std::env::args;

use cnxt::Colorize;
use rust_i18n::t;

use crate::{
    cli::{
        conf::handle_conf_command,
        go::handle_go_command,
        util::{
            HELP_ARGS, HelpSubcommand, build_help_subcommands, print_error_info,
        },
        version::handle_version_command,
    },
    config::CONFIG_COUNT,
    consts::EXE_NAME,
};

pub enum CliAction {
    RunConfig(Vec<String>),
    ManageConfig(ManageConfigAction),
    Version(VersionAction),
    ShowHelp(HelpInfo),
}

pub enum ManageConfigAction {
    List,
    New,
}

pub enum VersionAction {
    Show,
    Update,
}

pub enum HelpInfo {
    Main,
    Go,
    Conf,
    Version,
}

pub fn parse() -> Option<CliAction> {
    let args: Vec<String> = args().collect();
    let args_quantity = args.len();

    // Show help if no command is provided
    if args_quantity == 1 {
        return Some(CliAction::ShowHelp(HelpInfo::Main));
    }

    // Parse the first argument
    match args[1].as_str() {
        arg if HELP_ARGS.contains(&arg) => handle_help_command(args_quantity),
        "version" => handle_version_command(&args, args_quantity),
        "conf" => handle_conf_command(&args, args_quantity),
        "go" => handle_go_command(&args, args_quantity),
        _ => handle_unknown_command(),
    }
}

fn handle_help_command(args_quantity: usize) -> Option<CliAction> {
    if args_quantity == 2 {
        Some(CliAction::ShowHelp(HelpInfo::Main))
    } else {
        print_error_info(
            &[2],
            &t!("This command does not support more parameters"),
            Some(&t!("Remove extra parameters and try again")),
        );
        None
    }
}

// Handle unknown commands
fn handle_unknown_command() -> Option<CliAction> {
    print_error_info(
        &[1],
        &t!("This is not a valid command"),
        Some(&t!(
            "Type %{cmd} for help",
            cmd = format!("`{} help`", *EXE_NAME)
        )),
    );
    None
}

// Show the main help message
pub fn show_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("Usage:").bright_green(),
        EXE_NAME.bright_cyan(),
        t!("[command]").bright_yellow(),
        t!("[arguments]").bright_blue(),
    ));
    help.push(format!("{}", t!("Available commands:").bright_green()));
    let subcommands: Vec<HelpSubcommand> = vec![
        HelpSubcommand {
            name: "go",
            additional_arg: Some(t!("<config name>")),
            description: t!("Run specified configuration"),
        },
        HelpSubcommand {
            name: "conf",
            additional_arg: None,
            description: t!("Manage configuration files"),
        },
        HelpSubcommand {
            name: "version",
            additional_arg: None,
            description: t!("Show version information"),
        },
    ];
    help.extend(build_help_subcommands(subcommands));

    help.push(format!("\n{}:", t!("Examples").bright_green()));
    help.push(format!(
        "   {} {} {}",
        EXE_NAME.bright_cyan(),
        "go".bright_magenta(),
        "config".bright_yellow()
    ));
    help.push(format!(
        "   {}",
        t!("Run configuration named `config`").bright_magenta()
    ));

    help.push(format!("\n{}:", t!("Help").bright_green()));
    if *CONFIG_COUNT == 0 {
        help.push(format!(
            "   {}",
            t!(
                "For first time use, please run %{cmd} to create a new configuration",
                cmd = format!("`{} conf new`", *EXE_NAME)
            )
            .bright_yellow()
        ));
    }
    help.push(format!(
        "   {}",
        t!(
            "Online documentation %{url}",
            url = "https://solo.lance.fun"
        )
        .bright_white()
    ));

    for line in help {
        println!("{line}");
    }
}
