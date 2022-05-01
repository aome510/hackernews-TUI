use crate::prelude::*;
use crate::view;
use std::time::{Duration, SystemTime};
use substring::*;

const MAX_URL_LEN: usize = 50;

fn format_plural(amount: u64, time: &str) -> String {
    format!("{} {}{}", amount, time, if amount == 1 { "" } else { "s" })
}

fn get_time_offset_in_text(offset: u64) -> String {
    if offset < 60 {
        format_plural(offset, "second")
    } else if offset < 60 * 60 {
        format_plural(offset / 60, "minute")
    } else if offset < 60 * 60 * 24 {
        format_plural(offset / (60 * 60), "hour")
    } else if offset < 60 * 60 * 24 * 30 {
        format_plural(offset / (60 * 60 * 24), "day")
    } else if offset < 60 * 60 * 24 * 365 {
        format_plural(offset / (60 * 60 * 24 * 30), "month")
    } else {
        format_plural(offset / (60 * 60 * 24 * 365), "year")
    }
}

pub fn from_day_offset_to_time_offset_in_secs(day_offset: u32) -> u64 {
    let day_in_secs: u64 = 24 * 60 * 60;
    day_in_secs * (day_offset as u64)
}

/// Calculate the elapsed time and return the result
/// in an appropriate format depending on the duration
pub fn get_elapsed_time_as_text(time: u64) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let then = Duration::new(time, 0);
    let offset = now.as_secs() - then.as_secs();
    get_time_offset_in_text(offset)
}

/// A simple URL shortening function that reduces the
/// URL length if it exceeds a given threshold
pub fn shorten_url(url: &str) -> String {
    let len = url.chars().count();
    if len > MAX_URL_LEN {
        url.substring(0, 40).to_string() + "..." + url.substring(len - 10, len)
    } else {
        url.to_string()
    }
}

/// Construct a simple footer view
pub fn construct_footer_view<T: view::help_view::HasHelpView>() -> impl View {
    LinearLayout::horizontal()
        .child(
            TextView::new(StyledString::styled(
                "Hacker News Terminal UI - made by AOME ©",
                config::get_config_theme().component_style.bold,
            ))
            .align(align::Align::bot_center())
            .full_width(),
        )
        .child(
            LinearLayout::horizontal()
                .child(Button::new_raw(
                    format!("[{}: help] ", config::get_global_keymap().open_help_dialog),
                    |s| s.add_layer(T::construct_on_event_help_view()),
                ))
                .child(Button::new_raw("[back] ", |s| {
                    if s.screen_mut().len() > 1 {
                        s.pop_layer();
                    } else {
                        s.quit();
                    }
                }))
                .child(Button::new_raw("[quit] ", |s| s.quit())),
        )
}

/// Combine multiple styled strings into a single styled string
pub fn combine_styled_strings(strings: Vec<StyledString>) -> StyledString {
    strings.into_iter().fold(StyledString::new(), |mut acc, s| {
        acc.append(s);
        acc
    })
}

/// Construct a view's title bar
pub fn construct_view_title_bar(desc: &str) -> impl View {
    let style = config::get_config_theme().component_style.title_bar;
    Layer::with_color(
        TextView::new(StyledString::styled(desc, style))
            .h_align(align::HAlign::Center)
            .full_width(),
        style.into(),
    )
}

/// open a given url using a specific command
pub fn open_url_in_browser(url: &str) {
    if url.is_empty() {
        return;
    }

    let url = url.to_string();
    let url_open_command = &config::get_config().url_open_command;
    std::thread::spawn(move || {
        match std::process::Command::new(&url_open_command.command)
            .args(&url_open_command.options)
            .arg(&url)
            .output()
        {
            Err(err) => warn!(
                "failed to execute command `{} {}`: {}",
                url_open_command, url, err
            ),
            Ok(output) => {
                if !output.status.success() {
                    warn!(
                        "failed to execute command `{} {}`: {}",
                        url_open_command,
                        url,
                        std::str::from_utf8(&output.stderr).unwrap(),
                    )
                }
            }
        }
    });
}
