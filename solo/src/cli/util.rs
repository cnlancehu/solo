use std::{borrow::Cow, env::args};

use cnxt::Colorize as _;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr as _;

use crate::consts::EXE_NAME;

pub const HELP_ARGS: [&str; 3] = ["help", "--help", "-h"];

/// Print error information with highlighted arguments and suggestions
pub fn print_error_info(
    error_arg_nums: &[usize],
    error: &str,
    help: Option<&str>,
) {
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
        eprintln!("{line}");
    }
}

pub struct HelpSubcommand<'a> {
    pub name: &'a str,
    pub additional_arg: Option<Cow<'a, str>>,
    pub description: Cow<'a, str>,
}

impl HelpSubcommand<'_> {
    fn command_width(&self) -> usize {
        self.name.width()
            + self
                .additional_arg
                .as_ref()
                .map_or(0, |arg| 2 + arg.width())
    }
}

pub fn build_help_subcommands(
    mut subcommands: Vec<HelpSubcommand>,
) -> Vec<String> {
    let max_width = subcommands
        .iter()
        .map(HelpSubcommand::command_width)
        .max()
        .unwrap_or(0)
        + 4;

    subcommands.push(HelpSubcommand {
        name: "help",
        additional_arg: None,
        description: t!("Show this help message"),
    });

    subcommands
        .iter()
        .map(|s| {
            let arg_part =
                s.additional_arg.as_ref().map_or(String::new(), |arg| {
                    format!("  {}", arg.bright_yellow())
                });

            let padding = " ".repeat(max_width - s.command_width());

            format!(
                "   {}{}{}{}",
                s.name.bright_cyan(),
                arg_part,
                padding,
                s.description
            )
        })
        .collect()
}
