use std::{borrow::Cow, fs, path::Path, process::exit};

use anyhow::{Result, anyhow};
use cnxt::Colorize as _;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr as _;

use super::definition::{
    Config, ConfigFile, MACHINE_TYPES_WITH_OPTIONAL_SECRET_ID,
};
use crate::config::{
    definition::MACHINE_TYPES_WITH_OPTIONAL_REGION_ID, get_config_path,
};

pub fn get_config_list() -> Vec<ConfigFile> {
    let config_dir = &*super::CONFIG_DETECTION_PATH;
    let Ok(entries) = fs::read_dir(config_dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str()?;
            if Path::new(file_name_str)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
            {
                Some(file_name_str.to_string())
            } else {
                None
            }
        })
        .map(|filename| {
            let name = filename.trim_end_matches(".toml").to_string();
            ConfigFile { name, filename }
        })
        .collect()
}

pub fn process_config(config: Vec<String>) -> Result<Vec<Config>> {
    let mut configs = Vec::new();
    for name in config {
        let path = get_config_path(&name).ok_or_else(|| anyhow!(""))?;
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let str = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!(
                "{}",
                t!("Unable to read configuration file").bright_red()
            );
            eprintln!("{e}");
            exit(1);
        });
        let config: Config = toml::from_str(&str).map_err(|e| {
            let span = e.span().unwrap_or(0..0);
            print_config_error(
                filename,
                &str,
                span.start,
                span.end,
                &Cow::from(e.message().to_string()),
                None,
            );
            exit(1);
        })?;

        for server in &config.servers {
            if server.secret_id.is_empty() {
                if MACHINE_TYPES_WITH_OPTIONAL_SECRET_ID
                    .contains(&server.machine_type)
                {
                    continue;
                }
                eprintln!(
                    "{}",
                    t!("Configuration file contains errors").bright_red()
                );
                eprintln!(
                    "{}",
                    t!(
                        "Server %{name}'s secret_id cannot be empty",
                        name = server.name
                    )
                    .bright_red()
                );

                exit(1);
            }
            if server.region.is_empty() {
                if MACHINE_TYPES_WITH_OPTIONAL_REGION_ID
                    .contains(&server.machine_type)
                {
                    continue;
                }
                eprintln!(
                    "{}",
                    t!("Configuration file contains errors").bright_red()
                );
                eprintln!(
                    "{}",
                    t!(
                        "Server %{name}'s region cannot be empty",
                        name = server.name
                    )
                    .bright_red()
                );

                exit(1);
            }
        }

        configs.push(config);
    }
    Ok(configs)
}

struct ConfigContentLine {
    line_number: usize,
    content: String,
    is_error_line: bool,
}

fn print_config_error(
    file_name: &str,
    config_str: &str,

    start: usize,
    end: usize,
    message: &str,
    help: Option<Cow<'static, str>>,
) {
    eprintln!("{}", t!("Configuration file contains errors").bright_red());
    let (start_line, start_column) =
        index_to_line_and_column(config_str, start);
    let (end_line, end_column) = index_to_line_and_column(config_str, end);
    let indentation = end_line.to_string().width();
    let empty_line =
        format!("{}{}", " ".repeat(indentation + 1), "|".bright_cyan());

    eprintln!(
        "{}{}{}:{}:{}",
        " ".repeat(indentation),
        "--> ".bright_cyan(),
        file_name,
        start_line,
        start_column
    );
    eprintln!("{}", &empty_line);
    let content_lines: Vec<ConfigContentLine> = config_str
        .lines()
        .enumerate()
        .map(|(i, line)| ConfigContentLine {
            line_number: i + 1,
            content: line.to_string(),
            is_error_line: i + 1 >= start_line && i < end_line,
        })
        .collect();

    for content_line in &content_lines {
        if content_line.line_number == start_line - 1
            || content_line.line_number == end_line + 1
            || content_line.is_error_line
        {
            let line_number = content_line.line_number.to_string();
            if content_line.is_error_line {
                eprintln!(
                    "{}{} {} {}",
                    " ".repeat(indentation - line_number.width()),
                    line_number.bright_cyan(),
                    "|".bright_cyan(),
                    content_line.content.bright_yellow()
                );

                if start_line == end_line {
                    eprintln!(
                        "{} {}{}",
                        empty_line,
                        " ".repeat(start_column - 1),
                        "^".repeat(end_column - start_column).bright_red()
                    );
                }
            } else {
                eprintln!("{} {}", empty_line, content_line.content);
            }
        }
    }
    eprintln!("{}", &empty_line);
    eprintln!(
        "{}{}{}",
        " ".repeat(indentation + 1),
        "= ".bright_cyan(),
        message.bright_red()
    );
    if let Some(help) = help {
        eprintln!(
            "{}{}{}",
            " ".repeat(indentation + 1),
            "+ ".bright_cyan(),
            help.bright_green()
        );
    }
}

fn index_to_line_and_column(s: &str, index: usize) -> (usize, usize) {
    if index >= s.len() {
        let lines = s.lines().count();
        let last_line_len = s.lines().last().map_or(0, str::len);
        return (lines, last_line_len + 1);
    }

    let before = &s[..index];
    let line = before.chars().filter(|&c| c == '\n').count() + 1;

    let column = before
        .rfind('\n')
        .map_or_else(|| index + 1, |pos| index - pos);

    (line, column)
}
