use cnxt::Colorize as _;
use rust_i18n::t;

pub use super::config_new::new;
use crate::{
    cli::{
        CliAction, ManageConfigAction,
        util::{
            HELP_ARGS, HelpSubcommand, build_help_subcommands, print_error_info,
        },
    },
    config::reader::get_config_list,
    consts::EXE_NAME,
};

// Handle the `conf` command and its subcommands
pub fn handle_conf_command(
    args: &[String],
    args_quantity: usize,
) -> Option<CliAction> {
    match args_quantity {
        2 => Some(CliAction::ManageConfig(ManageConfigAction::ShowHelp)),
        3 => match args[2].as_str() {
            arg if HELP_ARGS.contains(&arg) => {
                Some(CliAction::ManageConfig(ManageConfigAction::ShowHelp))
            }

            "list" => Some(CliAction::ManageConfig(ManageConfigAction::List)),
            "new" => Some(CliAction::ManageConfig(ManageConfigAction::New)),

            _ => {
                print_error_info(
                    &[2],
                    &t!("Unknown configuration command"),
                    Some(&t!(
                        "Type %{cmd} for help",
                        cmd = format!("`{} conf help`", *EXE_NAME)
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

pub fn show_avaliable_configs() {
    let config_list = get_config_list();
    if config_list.is_empty() {
        println!("{}", t!("No configuration found").bright_red());
        println!(
            "{}",
            t!(
                "Run %{cmd} to create one",
                cmd = format!("`{} conf new`", *EXE_NAME)
            )
            .bright_red()
        );
        return;
    }

    println!("{}", t!("Available configurations:").bright_green());
    for f in &config_list {
        println!(
            "  {} {}",
            f.name.bright_yellow(),
            format!("({})", f.filename).bright_black(),
        );
    }
}

// Show the help message for the `conf` command
pub fn show_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("Usage:").bright_green(),
        EXE_NAME.bright_cyan(),
        "conf".bright_yellow(),
        t!("[command]").bright_blue()
    ));
    help.push(format!("{}", t!("Available commands:").bright_green()));
    let subcommands: Vec<HelpSubcommand> = vec![
        HelpSubcommand {
            name: "list",
            additional_arg: None,
            description: t!("List available configurations"),
        },
        HelpSubcommand {
            name: "new",
            additional_arg: None,
            description: t!("Create a new configuration"),
        },
    ];
    help.extend(build_help_subcommands(subcommands));

    for line in help {
        println!("{line}");
    }
}
