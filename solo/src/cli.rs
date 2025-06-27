use std::env::args;

use anyhow::{Result, anyhow};
use cnxt::Colorize;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

use crate::{config::CONFIG_LIST_NAMES, consts::EXE_NAME};

pub enum CliAction {
    RunConfig(Vec<String>),
    ManageConfig(ManageConfigAction),
    ShowVersion,
    ShowHelp,
}

pub enum ManageConfigAction {
    ShowHelp,
    List,
    New,
    Del,
    Edit,
}

pub fn parse() -> Result<CliAction> {
    let args: Vec<String> = args().collect();
    let args_quantity = args.len();

    // Show help if no command is provided
    if args_quantity == 1 {
        return Ok(CliAction::ShowHelp);
    }

    // Parse the first argument
    match args[1].as_str() {
        "version" | "help" => handle_simple_command(&args, args_quantity),
        "conf" => handle_conf_command(&args, args_quantity),
        "go" => handle_go_command(&args, args_quantity),
        _ => handle_unknown_command(),
    }
}

// Handle simple commands which do not require additional parameters
fn handle_simple_command(
    args: &[String],
    args_quantity: usize,
) -> Result<CliAction> {
    if args_quantity > 2 {
        print_error_info(
            &[2],
            &t!("This command does not support additional parameters"),
            Some(&t!("Remove them and try again")),
        );
        return Err(anyhow!("Extra parameters"));
    }

    Ok(match args[1].as_str() {
        "version" => CliAction::ShowVersion,
        "help" => CliAction::ShowHelp,
        _ => unreachable!(),
    })
}

// Handle the `conf` command and its subcommands
fn handle_conf_command(
    args: &[String],
    args_quantity: usize,
) -> Result<CliAction> {
    match args_quantity {
        2 => Ok(CliAction::ManageConfig(ManageConfigAction::ShowHelp)),
        3 => match args[2].as_str() {
            "help" => Ok(CliAction::ManageConfig(ManageConfigAction::ShowHelp)),
            "list" => Ok(CliAction::ManageConfig(ManageConfigAction::List)),
            "new" => Ok(CliAction::ManageConfig(ManageConfigAction::New)),
            "del" => Ok(CliAction::ManageConfig(ManageConfigAction::Del)),
            "edit" => Ok(CliAction::ManageConfig(ManageConfigAction::Edit)),

            _ => {
                print_error_info(
                    &[2],
                    &t!("Unknown configuration command"),
                    Some(&t!(
                        "Type %{cmd} for help",
                        cmd = format!("`{} conf help`", *EXE_NAME)
                    )),
                );
                Err(anyhow!("Missing parameter"))
            }
        },
        _ => {
            print_error_info(
                &[3],
                &t!("This command does not support more parameters"),
                Some(&t!("Remove extra parameters and try again")),
            );
            Err(anyhow!("Too many parameters"))
        }
    }
}

// Handle the `go` command
fn handle_go_command(
    args: &[String],
    args_quantity: usize,
) -> Result<CliAction> {
    if args_quantity == 2 {
        print_error_info(
            &[2],
            &t!("Missing configuration name"),
            Some(&t!("Please provide the configuration name to run")),
        );
        Err(anyhow!("Missing parameter"))
    } else {
        let configs: Vec<String> = args[2..].to_vec();

        // Check for duplicate configuration names
        let mut duplicates: Vec<(usize, &String)> = Vec::new();
        for i in 0..configs.len() {
            for j in i + 1..configs.len() {
                if configs[i] == configs[j]
                    && !duplicates.iter().any(|(_, name)| **name == configs[j])
                {
                    duplicates.push((j + 2, &configs[j]));
                }
            }
        }

        if !duplicates.is_empty() {
            print_error_info(
                &duplicates.iter().map(|(i, _)| *i).collect::<Vec<usize>>(),
                &t!("Duplicate configuration names detected"),
                Some(&t!("Please remove duplicate configuration names")),
            );
            return Err(anyhow!("Duplicate configuration names"));
        }

        let invalid_configs: Vec<(usize, &String)> = configs
            .iter()
            .enumerate()
            .filter(|(_, c)| !CONFIG_LIST_NAMES.contains(c))
            .map(|(i, c)| (2 + i, c))
            .collect();

        if invalid_configs.is_empty() {
            Ok(CliAction::RunConfig(configs))
        } else {
            print_error_info(
                &invalid_configs
                    .iter()
                    .map(|(i, _)| *i)
                    .collect::<Vec<usize>>(),
                &t!("Specified configuration not found"),
                Some(&t!("Please check if the configuration name is correct")),
            );

            if CONFIG_LIST_NAMES.is_empty() {
                println!(
                    "\n{}",
                    t!("No configuration files available").bright_red()
                );
                println!(
                    "{}",
                    t!(
                        "Run %{cmd} to create one",
                        cmd = format!("`{} conf new`", *EXE_NAME)
                    )
                    .bright_green()
                );
            } else {
                println!(
                    "\n{}",
                    t!(
                        "Run %{cmd} to view available configuration list",
                        cmd = format!("`{} conf list`", *EXE_NAME)
                    )
                    .bright_green()
                );
            }

            Err(anyhow!("Configuration not found"))
        }
    }
}

// Handle unknown commands
fn handle_unknown_command() -> Result<CliAction> {
    print_error_info(
        &[1],
        &t!("This is not a valid command"),
        Some(&t!(
            "Type %{cmd} for help",
            cmd = format!("`{} help`", *EXE_NAME)
        )),
    );
    Err(anyhow!("Unknown command"))
}

/// Print error information with highlighted arguments and suggestions
fn print_error_info(error_arg_nums: &[usize], error: &str, help: Option<&str>) {
    let args: Vec<String> = args().collect();
    let mut print_content = Vec::new();
    print_content.push(format!(
        "{}",
        t!("Command line argument error").bright_red().bold()
    ));

    print_content.push(format!(
        "{} {} {}",
        ">".bright_cyan(),
        EXE_NAME.as_str().bright_green(),
        args.iter()
            .skip(1)
            .enumerate()
            .map(|(i, arg)| {
                if error_arg_nums.contains(&(i + 1)) {
                    arg.bright_cyan().to_string()
                } else {
                    arg.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(" "),
    ));

    let mut space_before_error_arg = 2 + EXE_NAME.as_str().width();
    for (i, arg) in args.iter().skip(1).enumerate() {
        if error_arg_nums.contains(&(i + 1)) {
            break;
        }
        space_before_error_arg += arg.width() + 1;
    }

    let error_arg_start = error_arg_nums.iter().min().copied().unwrap_or(0);
    let mut space_after_error_arg = space_before_error_arg + 1;
    for arg in args.iter().skip(error_arg_start) {
        space_after_error_arg += 1 + arg.width();
    }

    let mut carets_line = " ".repeat(space_before_error_arg);
    for (i, arg) in args.iter().skip(error_arg_start).enumerate() {
        if error_arg_nums.contains(&(i + error_arg_start)) {
            carets_line.push(' ');
            for _ in 0..arg.width() {
                carets_line.push('^');
            }
        } else {
            carets_line.push_str(" ".repeat(arg.width()).as_str());
        }
    }

    print_content.push(format!("{} = {}", carets_line, error.bright_yellow()));

    if let Some(help) = help {
        print_content.push(format!(
            "{}+ {}",
            " ".repeat(space_after_error_arg),
            help.bright_green()
        ));
    }

    for line in print_content {
        println!("{line}");
    }
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
    help.push(help_print_subcommand(
        format!("go <{}>", t!("config_name")).as_str(),
        &t!("Run specified configuration"),
    ));
    help.push(help_print_subcommand(
        "conf",
        &t!("Manage configuration files"),
    ));
    help.push(help_print_subcommand(
        "version",
        &t!("Show version information"),
    ));
    help.push(help_print_subcommand("help", &t!("Show this help message")));

    help.push(format!("\n{}:", t!("Examples").bright_green()));
    help.push(format!(
        "   {} {} {}",
        EXE_NAME.bright_cyan(),
        "go".bright_magenta(),
        "solo-config".bright_yellow()
    ));
    help.push(format!(
        "   {}",
        t!("Run configuration named `solo-config`").bright_magenta()
    ));

    help.push(format!("\n{}:", t!("Help").bright_green()));
    // help.push(format!(
    //     "   {}",
    //     t!(
    //         "For first time use, please run %{cmd} to create a new configuration",
    //         cmd = format!("`{} conf new`", *EXE_NAME)
    //     )
    //     .bright_yellow()
    // ));
    help.push(format!(
        "   {}",
        t!(
            "Online documentation %{url}",
            url = "https://solo.lance.fun"
        )
    ));

    for line in help {
        println!("{line}");
    }
}

// Show the help message for the `conf` command
pub fn show_conf_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("Usage:").bright_green(),
        EXE_NAME.bright_cyan(),
        "conf".bright_yellow(),
        t!("[command]").bright_blue()
    ));
    help.push(format!("{}", t!("Available commands:").bright_green()));
    help.push(help_print_subcommand(
        "list",
        &t!("List available configurations"),
    ));
    help.push(help_print_subcommand(
        "new",
        &t!("Create a new configuration"),
    ));
    // help.push(help_print_subcommand("del", &t!("Delete a configuration")));
    // help.push(help_print_subcommand("edit", &t!("Edit a configuration")));
    help.push(help_print_subcommand("help", &t!("Show this help message")));

    for line in help {
        println!("{line}");
    }
}

// Helper function to format subcommand help lines
fn help_print_subcommand(subcommand: &str, description: &str) -> String {
    let reserve_space = 16 - subcommand.width();
    format!(
        "   {}{}{}",
        subcommand.bright_cyan(),
        " ".repeat(reserve_space),
        description
    )
}
