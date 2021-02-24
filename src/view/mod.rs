pub mod comment_view;
mod event_view;
pub mod story_view;
mod text_view;

use crate::prelude::*;

/// Calculate the elapsed time and result the result
/// in an appropriate format depending the duration
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
    } else {
        format!("{} days", elapsed_time_in_minutes / 60 / 24)
    }
}
