use std::{borrow::Cow, fs, path::Path, process::exit};

use anyhow::{Result, anyhow};
use cnxt::Colorize as _;
use json_spanned_value::{self as jsv, Spanned};
use rust_i18n::t;
use unicode_width::UnicodeWidthStr as _;
use walkdir::WalkDir;

use super::{
    CONFIG_DETECTION_PATH,
    definition::{Config, ConfigFile, Schedule},
};
use crate::{
    cli::EXE_NAME,
    config::{
        definition::{
            ConfigInput, MachineType, Notification, NotificationTrigger, Server,
        },
        get_config_path,
    },
    exec::ipfetcher::{EmbedIpProvider, IpProvider, Protocol},
};

pub struct ConfigReadError {
    pub start: usize,
    pub end: usize,
    pub message: Cow<'static, str>,
    pub help: Option<Cow<'static, str>>,
}

pub fn show_avaliable_configs() {
    let config_list = get_config_list();
    if config_list.is_empty() {
        println!("{}", t!("没有找到任何配置").bright_red());
        println!(
            "{}",
            t!(
                "运行 %{cmd} 创建一个",
                cmd = format!("`{} conf new`", *EXE_NAME)
            )
            .bright_red()
        );
        return;
    }

    println!("{}", t!("可用配置:").bright_green());
    for f in &config_list {
        println!(
            "  {} {}",
            f.name.bright_yellow(),
            format!("({})", f.filename).bright_black(),
        );
    }
}

pub fn get_config_list() -> Vec<ConfigFile> {
    let configfile_paths = WalkDir::new(&*CONFIG_DETECTION_PATH)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .filter(|n| {
            Path::new(n)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect::<Vec<String>>();
    configfile_paths
        .iter()
        .map(|path| {
            let name = path.trim_end_matches(".json").to_string();

            ConfigFile {
                name,
                filename: path.to_string(),
            }
        })
        .collect()
}

pub fn process_config(config: Vec<String>) -> Result<Vec<Config>> {
    let mut configs = Vec::new();
    for name in config {
        let config_file = get_config_path(&name).ok_or_else(|| anyhow!(""))?;
        let config_filename = config_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let config_str = fs::read_to_string(&config_file).unwrap_or_else(|e| {
            println!("{}", t!("无法读取配置文件").bright_red());
            println!("{e}");
            exit(1);
        });
        let config_input: ConfigInput =
            jsv::from_str(&config_str).map_err(|e| {
                print_config_json_parse_error(
                    config_filename,
                    &config_str,
                    e.line(),
                    e.column(),
                    &e.to_string(),
                );
                exit(1);
            })?;
        let config = parse_config(config_input).unwrap_or_else(|e| {
            print_config_error(
                config_file
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or(""),
                &config_str,
                &e,
            );
            exit(1);
        });
        configs.push(config);
    }
    Ok(configs)
}

pub fn parse_config(
    config_input: ConfigInput,
) -> Result<Config, Vec<ConfigReadError>> {
    let mut errors: Vec<ConfigReadError> = Vec::new();

    // Helper function to validate non-empty strings
    let validate_str = |field: &Spanned<Option<String>>,
                        error_msg: &'static str|
     -> (String, bool, Option<ConfigReadError>) {
        match &**field {
            Some(value) if !value.is_empty() => (value.clone(), false, None),
            _ => {
                let error = ConfigReadError {
                    start: field.start(),
                    end: field.end(),
                    message: t!(error_msg),
                    help: None,
                };
                (String::new(), true, Some(error))
            }
        }
    };

    // Check name field
    let (config_name, mut is_error_occurred, error) =
        validate_str(&config_input.name, "配置名称不能为空");
    if let Some(err) = error {
        errors.push(err);
    }

    // Check servers field
    let mut servers: Vec<Server> = Vec::new();
    match &*config_input.servers {
        Some(server_inputs) if !server_inputs.is_empty() => {
            for server in server_inputs {
                // Server name
                let (server_name, error, err_obj) =
                    validate_str(&server.name, "服务器名称不能为空");
                is_error_occurred |= error;
                if let Some(err) = err_obj {
                    errors.push(err);
                }

                // Machine ID
                let (machine_id, error, err_obj) =
                    validate_str(&server.machine_id, "服务器 ID 不能为空");
                is_error_occurred |= error;
                if let Some(err) = err_obj {
                    errors.push(err);
                }

                // Region
                let (region, error, err_obj) =
                    validate_str(&server.region, "服务器地域不能为空");
                is_error_occurred |= error;
                if let Some(err) = err_obj {
                    errors.push(err);
                }

                // Secret ID
                let (secret_id, error, err_obj) =
                    validate_str(&server.secret_id, "Secret ID 不能为空");
                is_error_occurred |= error;
                if let Some(err) = err_obj {
                    errors.push(err);
                }

                // Secret Key
                let (secret_key, error, err_obj) =
                    validate_str(&server.secret_key, "Secret Key 不能为空");
                is_error_occurred |= error;
                if let Some(err) = err_obj {
                    errors.push(err);
                }

                // Machine type
                let machine_type = if let Some(machine_type_str) =
                    &*server.machine_type
                {
                    match machine_type_str.to_lowercase().as_str() {
                        "qcloudvpc" | "qcloud_vpc" => MachineType::QcloudVpc,
                        "qcloudlighthouse" | "qcloud_lighthouse"
                        | "qcloudlh" | "qcloud_lh" => {
                            MachineType::QcloudLighthouse
                        }
                        "aliyunecs" | "aliyun_ecs" => MachineType::AliyunEcs,
                        "aliyunswas" | "aliyun_swas" => MachineType::AliyunSwas,
                        _ => {
                            errors.push(ConfigReadError {
                                start: server.machine_type.start(),
                                end: server.machine_type.end(),
                                message: t!("未知的机器类型"),
                                help: None,
                            });
                            is_error_occurred = true;
                            MachineType::QcloudVpc
                        }
                    }
                } else {
                    errors.push(ConfigReadError {
                        start: server.machine_type.start(),
                        end: server.machine_type.end(),
                        message: t!("机器类型不能为空"),
                        help: None,
                    });
                    is_error_occurred = true;
                    MachineType::QcloudVpc
                };

                // Protocol
                let protocol = if let Some(protocol_str) = &*server.protocol {
                    match protocol_str.to_lowercase().as_str() {
                        "ipv4" | "v4" => Protocol::V4,
                        "ipv6" | "v6" => Protocol::V6,
                        "both" => Protocol::Both,
                        _ => {
                            errors.push(ConfigReadError {
                                start: server.protocol.start(),
                                end: server.protocol.end(),
                                message: t!("未知的协议"),
                                help: None,
                            });
                            is_error_occurred = true;
                            Protocol::V4
                        }
                    }
                } else {
                    errors.push(ConfigReadError {
                        start: server.protocol.start(),
                        end: server.protocol.end(),
                        message: t!("协议不能为空"),
                        help: None,
                    });
                    is_error_occurred = true;
                    Protocol::V4
                };

                // Rules
                let rules = match &*server.rules {
                    Some(rules) if !rules.is_empty() => rules.clone(),
                    _ => {
                        errors.push(ConfigReadError {
                            start: server.rules.start(),
                            end: server.rules.end(),
                            message: t!("规则不能为空"),
                            help: None,
                        });
                        is_error_occurred = true;
                        Vec::new()
                    }
                };

                servers.push(Server {
                    machine_type,
                    machine_id,
                    name: server_name,
                    region,
                    secret_id,
                    secret_key,
                    protocol,
                    rules,
                });
            }
        }
        _ => {
            errors.push(ConfigReadError {
                start: config_input.servers.start(),
                end: config_input.servers.end(),
                message: t!("服务器字段不能为空"),
                help: None,
            });
            is_error_occurred = true;
        }
    }

    // Parse IP provider
    let ip_provider = if let Some(provider_input) = &*config_input.ip_provider {
        match (provider_input.embed.clone(), provider_input.url.clone()) {
            (Some(embed), None) => {
                if let Some(ip_provider) = EmbedIpProvider::from_str(&embed) {
                    IpProvider::Embed(ip_provider)
                } else {
                    errors.push(ConfigReadError {
                        start: config_input.ip_provider.start(),
                        end: config_input.ip_provider.end(),
                        message: t!("未知的 IP 提供商"),
                        help: None,
                    });
                    is_error_occurred = true;
                    IpProvider::Embed(EmbedIpProvider::MyExternalIp)
                }
            }
            (None, Some(url)) => IpProvider::Url(url),
            (None, None) => {
                errors.push(ConfigReadError {
                    start: config_input.ip_provider.start(),
                    end: config_input.ip_provider.end(),
                    message: t!("IP 提供商不能为空"),
                    help: None,
                });
                is_error_occurred = true;
                IpProvider::Embed(EmbedIpProvider::MyExternalIp)
            }
            (Some(_), Some(_)) => {
                errors.push(ConfigReadError {
                    start: config_input.ip_provider.start(),
                    end: config_input.ip_provider.end(),
                    message: t!("只能填写一个 IP 提供商"),
                    help: None,
                });
                is_error_occurred = true;
                IpProvider::Embed(EmbedIpProvider::MyExternalIp)
            }
        }
    } else {
        errors.push(ConfigReadError {
            start: config_input.ip_provider.start(),
            end: config_input.ip_provider.end(),
            message: t!("IP 提供商不能为空"),
            help: None,
        });
        is_error_occurred = true;
        IpProvider::Embed(EmbedIpProvider::MyExternalIp)
    };

    // Parse notifications
    let mut notifications: Vec<Notification> = Vec::new();
    if let Some(notification_inputs) = &*config_input.notifications {
        for notification in notification_inputs {
            if notification.name.is_empty() {
                errors.push(ConfigReadError {
                    start: notification.name.start(),
                    end: notification.name.end(),
                    message: t!("通知名称不能为空"),
                    help: None,
                });
                is_error_occurred = true;
            }

            let trigger = match notification.trigger.to_lowercase().as_str() {
                "onsuccess" => NotificationTrigger::OnSuccess,
                "onfailure" => NotificationTrigger::OnFailure,
                "onsuccessfullychanged" => {
                    NotificationTrigger::OnSuccessFullyChanged
                }
                "both" => NotificationTrigger::Both,
                _ => {
                    errors.push(ConfigReadError {
                        start: notification.trigger.start(),
                        end: notification.trigger.end(),
                        message: t!("未知的触发器"),
                        help: None,
                    });
                    is_error_occurred = true;
                    NotificationTrigger::OnSuccess
                }
            };

            notifications.push(Notification {
                name: (*notification.name).clone(),
                trigger,
                method: (*notification.method).clone(),
            });
        }
    }

    if is_error_occurred {
        return Err(errors);
    }

    Ok(Config {
        servers,
        schedule: config_input.schedule.unwrap_or(Schedule::Once),
        ip_provider,
        notifications,
        name: config_name,
    })
}

struct ConfigContentLine {
    line_number: usize,
    content: String,
    is_error_line: bool,
}

fn print_config_json_parse_error(
    file_name: &str,
    config_str: &str,
    line: usize,
    column: usize,
    message: &str,
) {
    println!("{}", t!("配置文件格式不正确").bright_red());

    let indentation = line.to_string().width();
    let empty_line =
        format!("{}{}", " ".repeat(indentation + 1), "|".bright_cyan());

    // Print error location
    println!(
        "{}{}{}:{}:{}",
        " ".repeat(indentation),
        "--> ".bright_cyan(),
        file_name,
        line,
        column
    );
    println!("{}", &empty_line);

    // Show context lines around error (one before, the error line, one after)
    let content_lines: Vec<&str> = config_str.lines().collect();
    let start_idx = line.saturating_sub(2);
    let end_idx = (line + 1).min(content_lines.len());

    for i in start_idx..end_idx {
        let line_number = i + 1;
        let line_content = content_lines.get(i).unwrap_or(&"");

        if line_number == line {
            // Error line
            println!(
                "{}{} {} {}",
                " ".repeat(indentation - line_number.to_string().width()),
                line_number.to_string().bright_cyan(),
                "|".bright_cyan(),
                line_content.bright_yellow()
            );
            // Error indicator
            println!(
                "{} {}{}",
                empty_line,
                " ".repeat(column.saturating_sub(1)),
                "^".repeat(line_content.width().saturating_sub(column) + 1)
                    .bright_red()
            );
        } else {
            println!(
                "{}{} {} {}",
                " ".repeat(indentation - line_number.to_string().width()),
                line_number.to_string().bright_cyan(),
                "|".bright_cyan(),
                line_content
            );
        }
    }

    println!("{}", &empty_line);
    println!(
        "{}{}{}",
        " ".repeat(indentation + 1),
        "= ".bright_cyan(),
        message.bright_red()
    );
}

fn print_config_error(
    file_name: &str,
    config_str: &str,
    errors: &[ConfigReadError],
) {
    println!("{}", t!("配置文件中存在错误").bright_red());
    for error in errors {
        let (start_line, start_column) =
            index_to_line_and_column(config_str, error.start);
        let (end_line, end_column) =
            index_to_line_and_column(config_str, error.end);
        let indentation = end_line.to_string().width();
        let empty_line =
            format!("{}{}", " ".repeat(indentation + 1), "|".bright_cyan());

        println!(
            "{}{}{}:{}:{}",
            " ".repeat(indentation),
            "--> ".bright_cyan(),
            file_name,
            start_line,
            start_column
        );
        println!("{}", &empty_line);
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
                    println!(
                        "{}{} {} {}",
                        " ".repeat(indentation - line_number.width()),
                        line_number.bright_cyan(),
                        "|".bright_cyan(),
                        content_line.content.bright_yellow()
                    );

                    if start_line == end_line {
                        println!(
                            "{} {}{}",
                            empty_line,
                            " ".repeat(start_column - 1),
                            "^".repeat(end_column - start_column).bright_red()
                        );
                    }
                } else {
                    println!("{} {}", empty_line, content_line.content);
                }
            }
        }
        println!("{}", &empty_line);
        println!(
            "{}{}{}",
            " ".repeat(indentation + 1),
            "= ".bright_cyan(),
            error.message.bright_red()
        );
        if let Some(help) = &error.help {
            println!(
                "{}{}{}",
                " ".repeat(indentation + 1),
                "+ ".bright_cyan(),
                help.bright_green()
            );
        }
    }
}

// More efficient implementation of index_to_line_and_column
fn index_to_line_and_column(s: &str, index: usize) -> (usize, usize) {
    if index >= s.len() {
        // Handle out of bounds index gracefully
        let lines = s.lines().count();
        let last_line_len = s.lines().last().map_or(0, str::len);
        return (lines, last_line_len + 1);
    }

    // Faster method: first split the string at the index
    let before = &s[..index];
    let line = before.chars().filter(|&c| c == '\n').count() + 1;

    // Find the last newline before the index
    let column = before
        .rfind('\n')
        .map_or_else(|| index + 1, |pos| index - pos);

    (line, column)
}
