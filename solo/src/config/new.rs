use std::{
    fs,
    io::{self, Write, stdout},
    time::Duration,
};

use anyhow::Result;
use cnxt::Colorize;
use crossterm::{
    cursor::{
        Hide, MoveToColumn, RestorePosition, SavePosition, SetCursorStyle, Show,
    },
    event::{Event, KeyCode, KeyModifiers, poll, read},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

use super::definition::{Config, MachineType, Schedule, Server};
use crate::{
    config::CONFIG_DETECTION_PATH,
    consts::EXE_NAME,
    ipfetcher::{EmbedIpProvider, IpProvider, Protocol},
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
            println!();
            println!("{}", t!("编辑它，然后使用以下命令运行").bright_blue());
            println!(
                "{} {} {}",
                EXE_NAME.bright_cyan(),
                "go".bright_magenta(),
                filename.trim_end_matches(FILE_SUFFIX).bright_green()
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
    execute!(stdout, SetCursorStyle::BlinkingBar, Hide)?;

    let raw_prompt = format!("{} {} ", t!("输入配置文件名"), ">");
    let prompt = format!(
        "{} {} ",
        t!("输入配置文件名").bright_cyan(),
        ">".bright_green()
    );
    let termsize = terminal::size().map_or(80, |size| size.0);
    let suffix = FILE_SUFFIX.bright_black();

    let mut buffer = String::new();

    print!("{prompt}{suffix}");
    queue!(stdout, MoveToColumn((raw_prompt.width()) as u16))?;
    stdout.flush()?;

    let result = input_loop(
        &mut stdout,
        &mut buffer,
        &prompt,
        &raw_prompt,
        &suffix,
        termsize,
    )?;

    cleanup_terminal(&mut stdout)?;

    Ok(result)
}

fn input_loop(
    stdout: &mut io::Stdout,
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

                if let Some(result) = handle_key_event(event, buffer) {
                    return Ok(result);
                }

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
            if event.is_press() && !buffer.is_empty() {
                buffer.pop();
            }
        }
        KeyCode::Enter => {
            if event.is_press() {
                if !buffer.is_empty() {
                    return Some(InputResult::Value(format!(
                        "{buffer}{FILE_SUFFIX}"
                    )));
                }
            }
        }
        _ => {}
    }

    None
}

fn update_display(
    stdout: &mut io::Stdout,
    prompt: &str,
    buffer: &str,
    suffix: &str,
    termsize: u16,
    raw_prompt: &str,
) -> Result<()> {
    let raw_prompt_width = raw_prompt.width();
    let buffer_width = buffer.width();
    let suffix_width = FILE_SUFFIX.width();

    queue!(stdout, SavePosition)?;

    queue!(stdout, MoveToColumn(0), Clear(ClearType::UntilNewLine))?;
    print!(
        "{}{}{}",
        prompt,
        buffer.bright_yellow(),
        if buffer.is_empty() {
            "".bright_black()
        } else {
            suffix.bright_black()
        }
    );

    // 计算并填充剩余空间
    let total_width = raw_prompt_width + buffer_width + suffix_width;
    if total_width < termsize as usize {
        queue!(stdout, Print(" ".repeat(termsize as usize - total_width)))?;
    }

    queue!(
        stdout,
        RestorePosition,
        MoveToColumn((raw_prompt_width + buffer_width) as u16)
    )?;

    stdout.flush()?;
    Ok(())
}

fn cleanup_terminal(stdout: &mut io::Stdout) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        stdout,
        Clear(ClearType::CurrentLine),
        MoveToColumn(0),
        SetCursorStyle::DefaultUserShape,
        Show
    )?;
    println!();

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
        no_proxy: None,
    };
    toml::to_string_pretty(&config).unwrap()
}
