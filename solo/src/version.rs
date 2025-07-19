use cnxt::Colorize as _;
use rust_i18n::t;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIME: &str = env!("SOLO_BUILD_TIME");
pub const TARGET_OS: &str = env!("SOLO_TARGET_OS");
pub const TARGET_ARCH: &str = env!("SOLO_TARGET_ARCH");
pub const TARGET_OS_DISPLAY: &str = env!("SOLO_TARGET_OS_DISPLAY");
pub const TARGET_ARCH_DISPLAY: &str = env!("SOLO_TARGET_ARCH_DISPLAY");

pub fn show() {
    println!(
        "{} {} {} {} {}",
        "Solo".bright_cyan(),
        format!("v{VERSION}").bright_green(),
        "-".bright_white(),
        TARGET_OS_DISPLAY.bright_magenta(),
        TARGET_ARCH_DISPLAY.bright_yellow()
    );
    println!(
        "{} {}",
        t!("Build at").bright_white(),
        BUILD_TIME.bright_blue()
    );
}
