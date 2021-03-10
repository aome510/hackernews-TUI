use crate::prelude::*;

const MAX_URL_LEN: usize = 64;

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

/// A simple URL shorten function that reduce the
/// URL length if exceeds a threshold
pub fn shorten_url(url: String) -> String {
    let len = url.chars().count();
    if len > MAX_URL_LEN {
        url.substring(0, 50).to_string() + "..." + url.substring(len - 14, len)
    } else {
        url
    }
}

/// Construct a simple footer view
pub fn construct_footer_view() -> impl View {
    let style = ColorStyle::new(
        Color::Dark(BaseColor::Black),
        Color::Light(BaseColor::White),
    );
    Layer::with_color(
        LinearLayout::horizontal()
            .child(
                TextView::new(StyledString::styled(
                    " Hacker News Terminal UI - made by AOME ©",
                    style,
                ))
                .align(align::Align::bot_left())
                .full_width(),
            )
            .child(
                TextView::new(StyledString::styled(" [?: help] ", style))
                    .align(align::Align::bot_right())
                    .full_width(),
            ),
        style,
    )
}
