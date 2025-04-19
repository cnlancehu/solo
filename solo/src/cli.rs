use std::env::{args, current_exe};

use anyhow::{Result, anyhow};
use cnxt::Colorize;
use lazy_static::lazy_static;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

use crate::config::CONFIG_LIST_NAMES;

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

lazy_static! {
    pub static ref EXE_NAME: String = {
        current_exe()
            .ok()
            .and_then(|path| {
                path.file_name().map(|n| n.to_string_lossy().into_owned())
            })
            .unwrap_or_else(|| "solo".into())
    };
}

pub fn parse() -> Result<CliAction> {
    let args: Vec<String> = args().collect();
    let args_quantity = args.len();

    // No arguments case
    if args_quantity == 1 {
        return Ok(CliAction::ShowHelp);
    }

    // Main command parsing
    match args[1].as_str() {
        "version" | "help" => handle_simple_command(&args, args_quantity),
        "conf" => handle_conf_command(&args, args_quantity),
        "go" => handle_go_command(&args, args_quantity),
        _ => handle_unknown_command(),
    }
}

fn handle_simple_command(
    args: &[String],
    args_quantity: usize,
) -> Result<CliAction> {
    // Commands that don't accept extra arguments
    if args_quantity > 2 {
        print_error_info(
            &[2],
            &t!("这个命令不支持附带参数"),
            Some(&t!("删除它，然后重试")),
        );
        return Err(anyhow!("多余参数"));
    }

    Ok(match args[1].as_str() {
        "version" => CliAction::ShowVersion,
        "help" => CliAction::ShowHelp,
        _ => unreachable!(),
    })
}

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
                    &t!("未知的配置命令"),
                    Some(&t!(
                        "输入 %{cmd} 查看帮助",
                        cmd = format!("`{} conf help`", *EXE_NAME)
                    )),
                );
                Err(anyhow!("缺少参数"))
            }
        },
        _ => {
            print_error_info(
                &[3],
                &t!("这个命令不支持附带更多的参数"),
                Some(&t!("删除多余参数，然后重试")),
            );
            Err(anyhow!("参数过多"))
        }
    }
}

fn handle_go_command(
    args: &[String],
    args_quantity: usize,
) -> Result<CliAction> {
    if args_quantity == 2 {
        print_error_info(
            &[2],
            &t!("缺少配置名称"),
            Some(&t!("请提供要运行的配置名称")),
        );
        Err(anyhow!("缺少参数"))
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
                &t!("检测到重复的配置名称"),
                Some(&t!("请移除重复的配置名称")),
            );
            return Err(anyhow!("重复的配置名称"));
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
                &t!("未找到指定的配置",),
                Some(&t!("请检查配置名称是否正确")),
            );

            if CONFIG_LIST_NAMES.is_empty() {
                println!("\n{}", t!("当前没有可用的配置文件").bright_red());
                println!(
                    "{}",
                    t!(
                        "运行 %{cmd} 创建一个",
                        cmd = format!("`{} conf new`", *EXE_NAME)
                    )
                    .bright_green()
                );
            } else {
                println!(
                    "\n{}",
                    t!(
                        "运行 %{cmd} 查看可用的配置文件列表",
                        cmd = format!("`{} conf list`", *EXE_NAME)
                    )
                    .bright_green()
                );
            }

            Err(anyhow!("未找到配置"))
        }
    }
}

fn handle_unknown_command() -> Result<CliAction> {
    print_error_info(
        &[1],
        &t!("这不是一个正确的命令"),
        Some(&t!(
            "输入 %{cmd} 查看帮助",
            cmd = format!("`{} help`", *EXE_NAME)
        )),
    );
    Err(anyhow!("未知命令"))
}

fn print_error_info(error_arg_nums: &[usize], error: &str, help: Option<&str>) {
    let args: Vec<String> = args().collect();
    let mut print_content = Vec::new();
    print_content
        .push(format!("{}", t!("命令行参数输入错误").bright_red().bold()));

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

pub fn show_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("使用方法:").bright_green(),
        EXE_NAME.bright_cyan(),
        t!("[命令]").bright_yellow(),
        t!("[参数]").bright_blue(),
    ));
    help.push(format!("{}", t!("可用命令:").bright_green()));
    help.push(help_print_subcommand(
        format!("go <{}>", t!("配置名称")).as_str(),
        &t!("运行指定配置"),
    ));
    help.push(help_print_subcommand("conf", &t!("管理配置文件")));
    help.push(help_print_subcommand("version", &t!("显示版本信息")));
    help.push(help_print_subcommand("help", &t!("显示此帮助信息")));

    help.push(format!("\n{}:", t!("示例").bright_green()));
    help.push(format!(
        "   {} {} {}",
        EXE_NAME.bright_cyan(),
        "go".bright_magenta(),
        "solo-config".bright_yellow()
    ));
    help.push(format!(
        "   {}",
        t!("运行名为 `solo-config` 的配置").bright_magenta()
    ));

    help.push(format!("\n{}:", t!("帮助").bright_green()));
    // help.push(format!(
    //     "   {}",
    //     t!(
    //         "初次使用，请先运行 %{cmd} 创建一个新的配置",
    //         cmd = format!("`{} conf new`", *EXE_NAME)
    //     )
    //     .bright_yellow()
    // ));
    help.push(format!(
        "   {}",
        t!("在线文档 %{url}", url = "https://solo.lance.fun")
    ));

    for line in help {
        println!("{line}");
    }
}

pub fn show_conf_help() {
    let mut help: Vec<String> = Vec::new();
    help.push(format!(
        "{} {} {} {}\n",
        t!("使用方法:").bright_green(),
        EXE_NAME.bright_cyan(),
        "conf".bright_yellow(),
        t!("[命令]").bright_blue()
    ));
    help.push(format!("{}", t!("可用命令:").bright_green()));
    help.push(help_print_subcommand("list", &t!("列出可用的配置")));
    // help.push(help_print_subcommand("new", &t!("创建一个新的配置")));
    // help.push(help_print_subcommand("del", &t!("删除一个配置")));
    // help.push(help_print_subcommand("edit", &t!("编辑一个配置")));
    help.push(help_print_subcommand("help", &t!("显示此帮助信息")));

    for line in help {
        println!("{line}");
    }
}

fn help_print_subcommand(subcommand: &str, description: &str) -> String {
    let reserve_space = 16 - subcommand.width();
    format!(
        "   {}{}{}",
        subcommand.bright_cyan(),
        " ".repeat(reserve_space),
        description
    )
}
