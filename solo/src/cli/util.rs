use std::env::args;

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
        println!("{line}");
    }
}

// Helper function to format subcommand help lines
pub fn help_print_subcommand(subcommand: &str, description: &str) -> String {
    let reserve_space = 16 - subcommand.width();
    let is_help = subcommand == "help";
    format!(
        "   {}{}{}",
        subcommand.bright_cyan().bright_black_if(is_help),
        " ".repeat(reserve_space),
        description.bright_black_if(is_help)
    )
}
