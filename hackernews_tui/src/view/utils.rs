use super::{article_view, help_view};
use crate::prelude::*;

/// Construct a simple footer view
pub fn construct_footer_view<T: help_view::HasHelpView>() -> impl View {
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

/// Open a given url using a specific command
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

pub fn open_ith_link_in_article_view(links: &[String], i: usize) -> Option<EventResult> {
    if i > 0 && i <= links.len() {
        let url = links[i - 1].clone();
        Some(EventResult::with_cb({
            move |s| article_view::add_article_view_layer(s, &url)
        }))
    } else {
        Some(EventResult::Consumed(None))
    }
}

pub fn open_ith_link_in_browser(links: &[String], i: usize) -> Option<EventResult> {
    if i > 0 && i <= links.len() {
        open_url_in_browser(&links[i - 1]);
        Some(EventResult::Consumed(None))
    } else {
        Some(EventResult::Consumed(None))
    }
}
