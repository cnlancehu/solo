use std::borrow::Cow;

use cnxt::Colorize;
use rust_i18n::t;

pub struct NotificationError<'a> {
    pub name: Cow<'a, str>,
    pub error: Cow<'a, str>,
}

#[must_use]
/// Generates a user-friendly error message from a vector of `NotificationError`.
pub fn explain_error(result: Vec<NotificationError<'_>>) -> Vec<String> {
    let mut error_message: Vec<String> = Vec::new();
    error_message
        .push(t!("Notification sending failed").bright_red().to_string());
    for msg in result {
        error_message.push(format!(
            "[{}] | {}",
            msg.name.bright_red(),
            msg.error.bright_red()
        ));
    }

    error_message
}
