use super::hn_client::*;
use cursive::{event::EventResult, traits::*, view::IntoBoxedView, views::*};
use log::warn;
use regex::Regex;

/// Construct a new Event view from a SelectView by adding
/// event handlers for a key pressed
fn construct_event_view<T: 'static>(view: SelectView<T>) -> OnEventView<SelectView<T>> {
    // add "j" and "k" for moving down and up the story list
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        })
}

/// Return a cursive's View from a story list
pub fn get_story_view(stories: Vec<Story>, hn_client: &HNClient) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();
    construct_event_view(
        SelectView::new()
            .with_all(
                stories
                    .into_iter()
                    .map(|story| (format!("{} ({})", story.title, story.by), story)),
            )
            .on_submit(move |s, story| {
                s.pop_layer();
                let comments = story.get_all_comments(&hn_client);
                s.add_layer(get_comment_view(comments, &hn_client));
            }),
    )
    .scrollable()
}

/// Parse a raw text from HN API to human-readable string
fn format_hn_text(s: String, link_re: &Regex) -> String {
    let s = htmlescape::decode_html(&s).unwrap_or(s);
    link_re
        .replace_all(&s.replace("<p>", "\n"), "$l")
        .to_string()
}

/// Return a cursive's View from a comment list
fn get_comment_view(comments: Vec<Comment>, hn_client: &HNClient) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();
    let link_re = Regex::new(r#"<a\s+?href=(?P<l>".+?").+?</a>"#).unwrap();

    OnEventView::new(LinearLayout::vertical().with(|v| {
        comments.into_iter().for_each(|comment| {
            v.add_child(Panel::new(TextView::new(format_hn_text(
                format!("{}: {}", comment.by, comment.text),
                &link_re,
            ))));
        })
    }))
    .on_event(cursive::event::Key::Backspace, move |s| {
        match hn_client.get_top_stories() {
            Ok(stories) => {
                s.pop_layer();
                s.add_layer(get_story_view(stories, &hn_client))
            }
            Err(err) => {
                warn!("failed to get top stories: {:#?}", err);
            }
        }
    })
    .scrollable()
}
