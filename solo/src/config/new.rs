use std::{
    fs,
    io::{Write, stdout},
    time::Duration,
};

use anyhow::Result;
use cnxt::Colorize;
use crossterm::{
    cursor::{Hide, MoveToColumn, SetCursorStyle, Show},
    event::{Event, KeyCode, KeyModifiers, poll, read},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

use super::definition::{Config, MachineType, Schedule, Server};
use crate::{
    config::CONFIG_DETECTION_PATH,
    exec::ipfetcher::{EmbedIpProvider, IpProvider, Protocol},
};

const DISALLOWED_CHARS: [char; 33] = [
    '/', ' ', ':', '*', '?', '"', '<', '>', '|', '\\', ',', '.', ';', '\'',
    '[', ']', '`', '~', '!', '@', '#', '$', '%', '^', '&', '(', ')', '_', '+',
    '{', '}', '-', '=',
];

const FILE_SUFFIX: &str = ".toml";
const POLL_DURATION: Duration = Duration::from_millis(200);

enum InputResult {
    Value(String),
    Cancelled,
}

pub fn new_config() -> Result<()> {
    println!("{}", t!("新建配置文件").bright_magenta());

    match get_config_filename()? {
        InputResult::Value(filename) => {
            let filename = format!("{filename}{FILE_SUFFIX}");
            let config = generate_example_config();
            let config_path = CONFIG_DETECTION_PATH.join(&filename);

            if config_path.exists() {
                println!("{}", t!("文件已存在").bright_red());
                return Ok(());
            }
            fs::write(&config_path, config).inspect_err(|_| {
                println!("{}", t!("无法创建配置文件").bright_red());
            })?;
            println!(
                "{}\n{} {}",
                t!("配置文件已创建").bright_green(),
                t!("位于").bright_cyan(),
                config_path.display().to_string().bright_yellow()
            );
        }
        InputResult::Cancelled => {
            println!("{}", t!("操作已取消").bright_red());
        }
    }

    Ok(())
}

fn get_config_filename() -> Result<InputResult> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, SetCursorStyle::BlinkingBar)?;

    let raw_prompt = format!("{} {} ", t!("输入配置文件名"), ">");
    let prompt = format!(
        "{} {} ",
        t!("输入配置文件名").bright_cyan(),
        ">".bright_green()
    );
    let termsize = terminal::size().map_or(80, |size| size.0);
    let suffix = FILE_SUFFIX.bright_black();

    let mut buffer = String::new();

    print!("{prompt}");
    stdout.flush()?;

    let result = input_loop(
        &mut stdout,
        &mut buffer,
        &prompt,
        &raw_prompt,
        &suffix,
        termsize,
    )?;

    // 清理并恢复终端状态
    cleanup_terminal(&mut stdout)?;

    Ok(result)
}

fn input_loop(
    stdout: &mut std::io::Stdout,
    buffer: &mut String,
    prompt: &str,
    raw_prompt: &str,
    suffix: &str,
    termsize: u16,
) -> Result<InputResult> {
    loop {
        if poll(POLL_DURATION)? {
            if let Event::Key(event) = read()? {
                execute!(stdout, Hide)?;

                // 处理按键输入
                if let Some(result) = handle_key_event(event, buffer) {
                    return Ok(result);
                }

                // 更新显示
                update_display(
                    stdout, prompt, buffer, suffix, termsize, raw_prompt,
                )?;
                execute!(stdout, Show)?;
            }
        }
    }
}

fn handle_key_event(
    event: crossterm::event::KeyEvent,
    buffer: &mut String,
) -> Option<InputResult> {
    match event.code {
        KeyCode::Char('c')
            if event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            return Some(InputResult::Cancelled);
        }
        KeyCode::Char(c) => {
            if event.is_press() && !DISALLOWED_CHARS.contains(&c) {
                buffer.push(c);
            }
        }
        KeyCode::Backspace => {
            if event.is_press() {
                buffer.pop();
            }
        }
        KeyCode::Enter => {
            if event.is_press() && !buffer.is_empty() {
                return Some(InputResult::Value(buffer.clone()));
            }
        }
        _ => {}
    }

    None
}

fn update_display(
    stdout: &mut std::io::Stdout,
    prompt: &str,
    buffer: &str,
    suffix: &str,
    termsize: u16,
    raw_prompt: &str,
) -> Result<()> {
    let show_suffix = !buffer.is_empty();
    let suffix_width = if show_suffix { suffix.width() } else { 0 };

    execute!(stdout, MoveToColumn(0))?;

    print!(
        "{}{}{}{}",
        prompt,
        buffer.bright_yellow(),
        if show_suffix {
            suffix.bright_black()
        } else {
            "".into()
        },
        " ".repeat(
            termsize as usize - prompt.width() - buffer.width() - suffix_width,
        ),
    );

    stdout.flush()?;

    execute!(
        stdout,
        MoveToColumn((raw_prompt.width() + buffer.width()) as u16),
    )?;

    Ok(())
}

fn cleanup_terminal(stdout: &mut std::io::Stdout) -> Result<()> {
    disable_raw_mode()?;
    println!();
    execute!(stdout, SetCursorStyle::DefaultUserShape, Show)?;

    Ok(())
}

fn generate_example_config() -> String {
    let config = Config {
        name: t!("配置文件名称").to_string(),
        servers: vec![Server {
            name: t!("服务器名称").to_string(),
            machine_type: MachineType::AliyunEcs,
            machine_id: t!("服务器实例ID").to_string(),
            region: t!("服务器地域").to_string(),
            secret_id: t!("密钥ID").to_string(),
            secret_key: t!("密钥Key").to_string(),
            protocol: Protocol::V4,
            rules: vec![
                t!("第一条规则").to_string(),
                t!("第二条规则").to_string(),
            ],
        }],
        schedule: Schedule::Once,
        ip_provider: IpProvider::Embed(EmbedIpProvider::IpEcho),
        notifications: vec![],
    };
    toml::to_string_pretty(&config).unwrap()
}
