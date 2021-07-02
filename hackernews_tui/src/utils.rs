use crate::prelude::*;
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
pub fn construct_footer_view<T: HasHelpView>() -> impl View {
    LinearLayout::horizontal()
        .child(
            TextView::new(StyledString::styled(
                "Hacker News Terminal UI - made by AOME ©",
                ColorStyle::front(PaletteColor::TitlePrimary),
            ))
            .align(align::Align::bot_center())
            .full_width(),
        )
        .child(
            LinearLayout::horizontal()
                .child(Button::new_raw(
                    format!("[{}: help] ", get_global_keymap().open_help_dialog),
                    |s| s.add_layer(T::construct_help_view()),
                ))
                .child(Button::new_raw("[quit] ", |s| s.quit())),
        )
}

/// Construct a status bar given a description text
pub fn get_status_bar_with_desc(desc: &str) -> impl View {
    Layer::with_color(
        TextView::new(StyledString::styled(
            desc,
            ColorStyle::new(
                PaletteColor::TitlePrimary,
                get_config_theme().status_bar_bg.color,
            ),
        ))
        .h_align(align::HAlign::Center)
        .full_width(),
        ColorStyle::back(get_config_theme().status_bar_bg.color),
    )
}

/// Construct StoryView based on the filtering tag
pub fn get_story_view_desc_by_tag(tag: &str) -> String {
    format!(
        "Story View - {}",
        match tag {
            "front_page" => "Front Page",
            "story" => "All Stories",
            "job" => "Jobs",
            "ask_hn" => "Ask HN",
            "show_hn" => "Show HN",
            _ => panic!("unknown tag: {}", tag),
        },
    )
}

/// open a given url using a specific command
pub fn open_url_in_browser(url: &str) {
    if url.is_empty() {
        return;
    }

    let url = url.to_string();
    let command = get_config().url_open_command.clone();
    std::thread::spawn(
        move || match std::process::Command::new(&command).arg(&url).output() {
            Err(err) => warn!("failed to execute command `{} {}`: {:?}", command, url, err),
            Ok(output) => {
                if !output.status.success() {
                    warn!(
                        "failed to execute command `{} {}`: {:?}",
                        command,
                        url,
                        std::str::from_utf8(&output.stderr).unwrap(),
                    )
                }
            }
        },
    );
}

/// check if a character is supported (if `allow_unicode` config option is disabled)
/// A character is considered to be supported if it is inside the ascii range or a "quote" character
pub fn allow_unicode_char(c: &char) -> bool {
    c.is_ascii()
        || *c == '\u{b4}'
        || *c == '\u{2018}'
        || *c == '\u{2019}'
        || *c == '\u{201c}'
        || *c == '\u{201d}'
}
