use cnxt::Colorize as _;
use rust_i18n::t;

use crate::{
    cli::{
        CliAction, HelpInfo,
        util::{HELP_ARGS, print_error_info},
    },
    config::CONFIG_LIST_NAMES,
    consts::EXE_NAME,
};

// Handle the `go` command
pub fn handle_go_command(
    args: &[String],
    args_quantity: usize,
) -> Option<CliAction> {
    if args_quantity == 2 {
        Some(CliAction::ShowHelp(HelpInfo::Go))
    } else {
        if let Some(arg) = args.get(2) {
            if HELP_ARGS.contains(&arg.as_str()) {
                return Some(CliAction::ShowHelp(HelpInfo::Go));
            }
        }

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
            return None;
        }

        let invalid_configs: Vec<(usize, &String)> = configs
            .iter()
            .enumerate()
            .filter(|(_, c)| !CONFIG_LIST_NAMES.contains(c))
            .map(|(i, c)| (2 + i, c))
            .collect();

        if invalid_configs.is_empty() {
            Some(CliAction::RunConfig(configs))
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

            None
        }
    }
}

// Show the help message for the `go` command
pub fn show_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("Usage:").bright_green(),
        EXE_NAME.bright_cyan(),
        "go".bright_yellow(),
        t!("<config name>").bright_blue()
    ));
    help.push(format!("{}:", t!("Examples").bright_green()));
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
    help.push(String::new());
    help.push(format!("   {}", t!("EXPERIMENTAL").bright_red()));
    help.push(format!(
        "   {} {} {} {}",
        EXE_NAME.bright_cyan(),
        "go".bright_magenta(),
        "config1".bright_yellow(),
        "config2".bright_yellow()
    ));
    help.push(format!(
        "   {}",
        t!("Run multiple configurations: `config1` and `config2`")
            .bright_magenta()
    ));

    for line in help {
        println!("{line}");
    }
}
