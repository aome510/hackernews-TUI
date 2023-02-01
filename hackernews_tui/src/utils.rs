use crate::prelude::*;
use std::time::{Duration, SystemTime};

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
    let chars = url.chars().collect::<Vec<_>>();
    let len = chars.len();
    if len > 50 {
        String::from_iter(chars[..40].iter()) + "..." + &String::from_iter(chars[len - 10..].iter())
    } else {
        url.to_string()
    }
}
/// Combine multiple styled strings into a single styled string
pub fn combine_styled_strings(strings: Vec<StyledString>) -> StyledString {
    strings.into_iter().fold(StyledString::new(), |mut acc, s| {
        acc.append(s);
        acc
    })
}

/// decode a HTML encoded string
pub fn decode_html(s: &str) -> String {
    html_escape::decode_html_entities(s).into()
}
