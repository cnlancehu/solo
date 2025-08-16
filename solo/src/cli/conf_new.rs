use std::{
    fs,
    io::{Write, stdout},
    time::Duration,
};

use anyhow::Result;
use cnxt::Colorize;
use crossterm::{
    cursor::{
        MoveToColumn, RestorePosition, SavePosition, SetCursorStyle, Show,
    },
    event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

use crate::{
    config::{
        CONFIG_DETECTION_PATH,
        definition::{Config, MachineType, Schedule, Server},
    },
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

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), SetCursorStyle::BlinkingBar, Show)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            Clear(ClearType::CurrentLine),
            MoveToColumn(0),
            SetCursorStyle::DefaultUserShape,
            Show
        );
        println!();
    }
}

pub fn new() {
    println!("{}", t!("Create Config File").bright_magenta());

    match get_config_filename() {
        Ok(InputResult::Value(filename)) => create_config_file(filename),
        Ok(InputResult::Cancelled) => {
            println!("{}", t!("Operation cancelled").bright_red());
        }
        Err(e) => println!(
            "{}",
            t!(
                "Error creating config file: %{error}",
                error = e.to_string()
            )
            .bright_red()
        ),
    }
}

fn create_config_file(filename: String) {
    let config_path = CONFIG_DETECTION_PATH.join(&filename);
    if config_path.exists() {
        println!("{}", t!("File already exists").bright_red());
        return;
    }

    if fs::write(&config_path, generate_example_config()).is_err() {
        println!("{}", t!("Unable to create config file").bright_red());
        return;
    }

    println!(
        "{}\n{} {}",
        t!("Config file created").bright_green(),
        t!("Located at").bright_cyan(),
        config_path.display().to_string().bright_yellow()
    );
    println!(
        "\n{}",
        t!("Edit it, then run with the following command").bright_blue()
    );
    println!(
        "{} {} {}",
        EXE_NAME.bright_cyan(),
        "go".bright_magenta(),
        filename.trim_end_matches(FILE_SUFFIX).bright_green()
    );
}

fn get_config_filename() -> Result<InputResult> {
    let _guard = TerminalGuard::new()?;

    let raw_prompt = format!("{} > ", t!("Enter config file name"));
    let prompt = format!(
        "{} {} ",
        t!("Enter config file name").bright_cyan(),
        ">".bright_green()
    );
    let termsize = terminal::size().map_or(80, |size| size.0);

    let mut buffer = String::new();
    render_prompt(
        &prompt,
        &buffer,
        &FILE_SUFFIX.bright_black(),
        termsize,
        &raw_prompt,
    )?;

    loop {
        if poll(POLL_DURATION)?
            && let Event::Key(event) = read()?
        {
            if let Some(res) = handle_key_event(event, &mut buffer) {
                return Ok(res);
            }
            render_prompt(
                &prompt,
                &buffer,
                &FILE_SUFFIX.bright_black(),
                termsize,
                &raw_prompt,
            )?;
        }
    }
}

fn handle_key_event(
    event: KeyEvent,
    buffer: &mut String,
) -> Option<InputResult> {
    match event.code {
        KeyCode::Char('c')
            if event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            Some(InputResult::Cancelled)
        }
        KeyCode::Char(c)
            if event.is_press() && !DISALLOWED_CHARS.contains(&c) =>
        {
            buffer.push(c);
            None
        }
        KeyCode::Backspace if event.is_press() && !buffer.is_empty() => {
            buffer.pop();
            None
        }
        KeyCode::Enter if event.is_press() && !buffer.is_empty() => {
            Some(InputResult::Value(format!("{buffer}{FILE_SUFFIX}")))
        }
        _ => None,
    }
}

fn render_prompt(
    prompt: &str,
    buffer: &str,
    suffix: &impl ToString,
    termsize: u16,
    raw_prompt: &str,
) -> Result<()> {
    let binding = suffix.to_string();
    let suffix_colored = binding.bright_black();

    let mut out = stdout();
    queue!(
        out,
        SavePosition,
        MoveToColumn(0),
        Clear(ClearType::UntilNewLine)
    )?;
    print!(
        "{}{}{}",
        prompt,
        buffer.bright_yellow(),
        if buffer.is_empty() {
            "".bright_black()
        } else {
            suffix_colored
        }
    );

    let total_width = raw_prompt.width() + buffer.width() + FILE_SUFFIX.width();
    if total_width < termsize as usize {
        queue!(out, Print(" ".repeat(termsize as usize - total_width)))?;
    }
    queue!(
        out,
        RestorePosition,
        MoveToColumn((raw_prompt.width() + buffer.width()) as u16)
    )?;
    out.flush()?;
    Ok(())
}

fn generate_example_config() -> String {
    let config = Config {
        name: t!("Config Name").to_string(),
        servers: vec![Server {
            name: t!("Server Name").to_string(),
            machine_type: MachineType::AliyunEcs,
            machine_id: t!("Server Instance ID").to_string(),
            region: t!("Server Region").to_string(),
            secret_id: t!("Secret ID").to_string(),
            secret_key: t!("Secret Key").to_string(),
            protocol: Protocol::V4,
            rules: vec![
                t!("First Rule").to_string(),
                t!("Second Rule").to_string(),
            ],
        }],
        schedule: Schedule::Once,
        ip_provider: IpProvider::Embed(EmbedIpProvider::IpEcho),
        notifications: vec![],
        no_proxy: None,
    };
    toml::to_string_pretty(&config).unwrap()
}
