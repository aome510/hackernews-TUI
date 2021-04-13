use std::time::{Duration, SystemTime};
use substring::*;

use super::theme::*;

use crate::prelude::*;

const MAX_URL_LEN: usize = 64;

/// Calculate the elapsed time and return the result
/// in an appropriate format depending on the duration
pub fn get_elapsed_time_as_text(time: u64) -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let then = Duration::new(time, 0);
    let elapsed_time_in_minutes = (now.as_secs() - then.as_secs()) / 60;
    if elapsed_time_in_minutes < 60 {
        format!("{} minutes", elapsed_time_in_minutes)
    } else if elapsed_time_in_minutes < 60 * 24 {
        format!("{} hours", elapsed_time_in_minutes / 60)
    } else if elapsed_time_in_minutes < 60 * 24 * 365 {
        format!("{} days", elapsed_time_in_minutes / 60 / 24)
    } else {
        format!("{} years", elapsed_time_in_minutes / 60 / 24 / 365)
    }
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
pub fn construct_footer_view<T: HasHelpView>(client: &hn_client::HNClient) -> impl View {
    LinearLayout::horizontal()
        .child(
            TextView::new(StyledString::styled(
                "Hacker News Terminal UI - made by AOME ©",
                ColorStyle::front(TEXT_BOLD_COLOR),
            ))
            .align(align::Align::bot_center())
            .full_width(),
        )
        .child(
            LinearLayout::horizontal()
                .child(Button::new_raw("[front page] ", {
                    let client = client.clone();
                    move |s| story_view::add_story_view_layer(s, &client)
                }))
                .child(Button::new_raw("[search] ", {
                    let client = client.clone();
                    move |s| search_view::add_search_view_layer(s, &client)
                }))
                .child(Button::new_raw("[help] ", |s| {
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
            ColorStyle::new(TEXT_BOLD_COLOR, STATUS_BAR_COLOR),
        ))
        .h_align(align::HAlign::Center)
        .full_width(),
        ColorStyle::back(STATUS_BAR_COLOR),
    )
}
