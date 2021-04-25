use std::time::{Duration, SystemTime};
use substring::*;

use crate::prelude::*;

const MAX_URL_LEN: usize = 64;

fn format_plural(amount: u64, time: &str) -> String {
    format!("{} {}{}", amount, time, if amount == 1 { "" } else { "s" })
}

fn get_offset_time_as_text(offset: u64) -> String {
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

/// Calculate the elapsed time and return the result
/// in an appropriate format depending on the duration
pub fn get_elapsed_time_as_text(time: u64) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let then = Duration::new(time, 0);
    let offset = now.as_secs() - then.as_secs();
    get_offset_time_as_text(offset)
}

/// A simple URL shortening function that reduces the
/// URL length if it exceeds a given threshold
pub fn shorten_url(url: &str) -> String {
    let len = url.chars().count();
    if len > MAX_URL_LEN {
        url.substring(0, 50).to_string() + "..." + url.substring(len - 14, len)
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
                .child(Button::new_raw("[?/<ctrl-h>: help] ", |s| {
                    s.add_layer(T::construct_help_view())
                }))
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
pub fn get_story_view_desc_by_tag(
    tag: &str,
    by_date: bool,
    page: usize,
    time_offset_in_secs: Option<u64>,
) -> String {
    format!(
        "Story View - {} (sort_by: {}, time_range: {}, page: {})",
        match tag {
            "front_page" => "Front Page",
            "story" => "All Stories",
            "job" => "Jobs",
            "ask_hn" => "Ask HN",
            "show_hn" => "Show HN",
            _ => panic!("unknown tag: {}", tag),
        },
        if by_date { "date" } else { "popularity" },
        match time_offset_in_secs {
            None => "all time".to_string(),
            Some(time_offset_in_secs) =>
                "past ".to_owned() + &get_offset_time_as_text(time_offset_in_secs),
        },
        page + 1
    )
}
