use std::{borrow::Cow, sync::Arc};

use anyhow::Error;
use cnxt::Colorize;
use rust_i18n::t;
use solo_lib::SdkError;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct ExecutionReport<'a> {
    pub id: usize,
    pub config_name: Cow<'a, str>,
    pub finished_timestamp: i64,

    pub ip_fetching_status: ExecutionReportIpFetching<'a>,
    pub server_status: Vec<ExecutionReportServer<'a>>,
}

#[derive(Debug, Clone)]
pub enum ExecutionReportIpFetching<'a> {
    Success {
        ipv4: Cow<'a, str>,
        ipv6: Cow<'a, str>,
    },
    Failed {
        error: Arc<Error>,
    },
}

#[derive(Debug, Clone)]
pub struct ExecutionReportServer<'a> {
    pub name: Cow<'a, str>,
    pub status: ExecutionReportServerStatus<'a>,
}

#[derive(Debug, Clone)]
pub enum ExecutionReportServerStatus<'a> {
    Success {
        is_ip_changed: bool,
    },
    Failed {
        error: Arc<Error>,
        when: Cow<'a, str>,
    },
}

pub fn show_brief_report(report: &ExecutionReport, color: bool) -> Vec<String> {
    let mut content: Vec<String> = Vec::new();
    if let ExecutionReportIpFetching::Failed { error } =
        &report.ip_fetching_status
    {
        content.push(format!(
            "{} | {}",
            t!("获取 IP 失败").bright_red_if(color),
            explain_error(error, color).join("\n")
        ));
    }

    for server_status in report.server_status.clone() {
        match server_status.status {
            ExecutionReportServerStatus::Success { is_ip_changed } => {
                if is_ip_changed {
                    content.push(format!(
                        "[{}] {}",
                        server_status.name.bright_green_if(color),
                        t!("IP 更改成功").bright_green_if(color)
                    ));
                } else {
                    content.push(format!(
                        "[{}] {}",
                        server_status.name.bright_green_if(color),
                        t!("IP 相同").bright_green_if(color)
                    ));
                }
            }
            ExecutionReportServerStatus::Failed { error, when } => {
                let indent_width = server_status.name.width() + 2;
                content.push(format!(
                    "[{}] | {}",
                    server_status.name.bright_red_if(color),
                    t!("在 %{when} 时出错", when = when).bright_red_if(color)
                ));
                for line in explain_error(&error, true) {
                    content.push(format!(
                        "{:indent_width$} | {}",
                        "",
                        line,
                        indent_width = indent_width
                    ));
                }
            }
        }
    }
    content
}

pub fn show_full_report(
    report: &ExecutionReport,
    color: bool,
    show_ipaddr: bool,
) -> Vec<String> {
    let mut content: Vec<String> = Vec::new();
    content.push(format!(
        "{} | {}",
        t!("配置文件").bright_green_if(color),
        report.config_name.bright_green_if(color)
    ));
    content.push(format!(
        "{} | {}",
        t!("运行于").bright_green_if(color),
        chrono::DateTime::from_timestamp(report.finished_timestamp, 0)
            .map_or_else(
                || report.finished_timestamp.to_string(),
                |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()
            )
            .bright_green_if(color)
    ));
    if show_ipaddr {
        match &report.ip_fetching_status {
            ExecutionReportIpFetching::Success { ipv4, ipv6 } => {
                content.push(format!(
                    "{} | {}",
                    t!("IP 地址").bright_green_if(color),
                    vec![ipv4.as_ref(), ipv6.as_ref()]
                        .into_iter()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<&str>>()
                        .join(" / ")
                        .bright_green_if(color)
                ));
            }
            ExecutionReportIpFetching::Failed { error } => {
                content.push(format!(
                    "{} | {}",
                    t!("获取 IP 失败").bright_red_if(color),
                    explain_error(error, color).join("\n")
                ));
            }
        }
    }

    for server_status in report.server_status.clone() {
        match server_status.status {
            ExecutionReportServerStatus::Success { is_ip_changed } => {
                if is_ip_changed {
                    content.push(format!(
                        "[{}] {}",
                        server_status.name.bright_green_if(color),
                        t!("IP 更改成功").bright_green_if(color)
                    ));
                } else {
                    content.push(format!(
                        "[{}] {}",
                        server_status.name.bright_green_if(color),
                        t!("IP 相同").bright_green_if(color)
                    ));
                }
            }
            ExecutionReportServerStatus::Failed { error, when: _ } => {
                let indent_width = server_status.name.width() + 2;
                content.push(format!(
                    "[{}] | {}",
                    server_status.name.bright_red_if(color),
                    t!("运行失败").bright_red_if(color)
                ));
                for line in explain_error(&error, color) {
                    content.push(format!(
                        "{:indent_width$} | {}",
                        "",
                        line,
                        indent_width = indent_width
                    ));
                }
            }
        }
    }
    content
}

fn explain_error(error: &Error, color: bool) -> Vec<String> {
    if let Some(sdkerror) = error.downcast_ref::<SdkError>() {
        explain_sdkerror(sdkerror, color)
    } else if let Some(reqwest_error) = error.downcast_ref::<reqwest::Error>() {
        vec![format!(
            "{} | {}",
            t!("网络请求错误"),
            reqwest_error.to_string().bright_red_if(color)
        )]
    } else if let Some(serde_json_error) =
        error.downcast_ref::<serde_json::Error>()
    {
        vec![format!(
            "{} | {}",
            t!("内容解析错误"),
            serde_json_error.to_string().bright_red_if(color)
        )]
    } else {
        vec![format!(
            "{} | {}",
            t!("未知错误"),
            error.to_string().bright_red_if(color)
        )]
    }
}

fn explain_sdkerror(error: &SdkError, color: bool) -> Vec<String> {
    let indent_width = [
        t!("请求 ID").width(),
        t!("错误代码").width(),
        t!("错误信息").width(),
    ]
    .iter()
    .max()
    .unwrap_or(&0)
        + 2;
    let mut error_message: Vec<String> = Vec::new();
    error_message.push(
        t!("在向云服务商发送请求时发生错误")
            .bright_red_if(color)
            .to_string(),
    );
    error_message.push(format!(
        "{} | {}",
        calc_indent_content(&t!("请求 ID"), indent_width),
        error.request_id.bright_red_if(color),
    ));
    error_message.push(format!(
        "{} | {}",
        calc_indent_content(&t!("错误代码"), indent_width),
        error.code.bright_red_if(color),
    ));
    {
        let lines: Vec<&str> = error.message.lines().collect();
        if let Some(first_line) = lines.first() {
            error_message.push(format!(
                "{} | {}",
                calc_indent_content(&t!("错误信息"), indent_width),
                first_line.bright_red_if(color),
            ));
            for line in lines.iter().skip(1) {
                error_message.push(format!(
                    "{:indent_width$} | {}",
                    "",
                    line.bright_red_if(color),
                    indent_width = indent_width
                ));
            }
        }
    }
    error_message
}

fn calc_indent_content(content: &str, indent_width: usize) -> String {
    format!("{}{}", " ".repeat(indent_width - content.width()), content)
}
