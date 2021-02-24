use super::hn_client::*;
use anyhow::Result;
use cursive::{event::EventResult, traits::*, view::IntoBoxedView, views::*};
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
            .with_all(stories.into_iter().enumerate().map(|(i, story)| {
                (
                    format!(
                        "{}. {} (author: {}, {} comments, {} points)",
                        i,
                        story.title.clone().unwrap_or("unknown title".to_string()),
                        story.author.clone().unwrap_or("unknown_user".to_string()),
                        story.num_comments,
                        story.points
                    ),
                    story,
                )
            }))
            .on_submit(move |s, story| match get_comment_view(story, &hn_client) {
                Err(err) => {
                    log::error!("failed to construct comment view: {:#?}", err);
                }
                Ok(comment_view) => {
                    s.pop_layer();
                    s.add_layer(comment_view);
                }
            }),
    )
    .scrollable()
}

/// Parse a raw text from HN API to human-readable string
fn format_hn_text(
    s: String,
    paragraph_re: &Regex,
    italic_re: &Regex,
    code_re: &Regex,
    link_re: &Regex,
) -> String {
    let mut s = htmlescape::decode_html(&s).unwrap_or(s);
    s = paragraph_re.replace_all(&s, "${paragraph}\n").to_string();
    s = link_re.replace_all(&s, "${link}").to_string();
    s = italic_re.replace_all(&s, "*${text}*").to_string();
    s = code_re.replace_all(&s, "```\n${code}\n```").to_string();
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
    let paragraph_re = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
    let italic_re = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
    let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
    let link_re = Regex::new(r#"<a\s+?href=(?P<link>".+?").+?</a>"#).unwrap();

    comments
        .par_iter()
        .flat_map(|comment| {
            let comment = &comment.as_ref();
            let mut subcomments = parse_comment_text_list(&comment.children, height + 1);
            let first_subcomment = (
                format!(
                    "{} {} ago\n{}",
                    comment.author.clone().unwrap_or("unknown_user".to_string()),
                    get_elapsed_time_as_text(comment.time),
                    format_hn_text(
                        comment
                            .text
                            .clone()
                            .unwrap_or("---deleted comment---".to_string()),
                        &paragraph_re,
                        &italic_re,
                        &code_re,
                        &link_re,
                    )
                ),
                height,
            );
            subcomments.insert(0, first_subcomment);
            subcomments
        })
        .collect()
}

/// Return a cursive's View from a comment list
fn get_comment_view(story: &Story, hn_client: &HNClient) -> Result<impl IntoBoxedView> {
    let hn_client = hn_client.clone();

    let comments = parse_comment_text_list(&story.get_comments(&hn_client)?, 0);

    Ok(OnEventView::new(
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
                log::error!("failed to get top stories: {:#?}", err);
            }
        }
    }))
}
