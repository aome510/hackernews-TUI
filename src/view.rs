use super::hn_client::*;
use cursive::{event::EventResult, traits::*, view::IntoBoxedView, views::*};
use log::warn;
use rayon::prelude::*;
use regex::Regex;
use std::time::{Duration, SystemTime};

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
fn format_hn_text(s: String, italic_re: &Regex, code_re: &Regex, link_re: &Regex) -> String {
    let mut s = htmlescape::decode_html(&s).unwrap_or(s);
    s = link_re
        .replace_all(&s.replace("<p>", "\n"), "${link}")
        .to_string();
    s = italic_re.replace_all(&s, "${text}").to_string();
    s = code_re.replace_all(&s, "${code}").to_string();
    s
}

/// Calculate the elapsed time and result the result
/// in an appropriate format depending the duration
fn get_elapsed_time_as_text(time: u64) -> String {
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

/// Retrieve all comments recursively and parse them into readable texts
fn parse_comment_text_list(comments: &Vec<Box<Comment>>, height: usize) -> Vec<(String, usize)> {
    let italic_re = Regex::new(r"<i>(?P<text>.+?)</i>").unwrap();
    let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)</code></pre>").unwrap();
    let link_re = Regex::new(r#"<a\s+?href=(?P<link>".+?").+?</a>"#).unwrap();

    comments
        .par_iter()
        .flat_map(|comment| {
            let comment = &comment.as_ref();
            let mut comments = parse_comment_text_list(&comment.subcomments, height + 1);
            comments.insert(
                0,
                (
                    format_hn_text(
                        format!(
                            "{} {} ago\n{}",
                            comment.by,
                            get_elapsed_time_as_text(comment.time),
                            comment.text
                        ),
                        &italic_re,
                        &code_re,
                        &link_re,
                    ),
                    height,
                ),
            );
            comments
        })
        .collect()
}

/// Return a cursive's View from a comment list
fn get_comment_view(comments: Vec<Comment>, hn_client: &HNClient) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();

    let comments = parse_comment_text_list(
        &comments
            .into_iter()
            .map(|comment| Box::new(comment))
            .collect(),
        0,
    );

    OnEventView::new(
        LinearLayout::vertical()
            .with(|v| {
                comments.into_iter().for_each(|comment| {
                    v.add_child(PaddedView::lrtb(
                        comment.1 * 2,
                        0,
                        0,
                        1,
                        TextView::new(comment.0),
                    ));
                })
            })
            .scrollable(),
    )
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
}
