use super::event_view;
use super::text_view;
use crate::prelude::*;

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

/// Retrieve all comments recursively and parse them into readable texts
fn parse_comment_text_list(
    comments: &Vec<Box<hn_client::Comment>>,
    height: usize,
) -> Vec<(String, usize)> {
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
                    comment
                        .author
                        .clone()
                        .unwrap_or("-unknown_user-".to_string()),
                    super::get_elapsed_time_as_text(comment.time),
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
pub fn get_comment_view(
    comments: &Vec<Box<hn_client::Comment>>,
    hn_client: &hn_client::HNClient,
) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();

    let comments = parse_comment_text_list(&comments, 0);
    let heights = comments
        .iter()
        .map(|comment| comment.1)
        .collect::<Vec<usize>>();

    event_view::construct_event_view(LinearLayout::vertical().with(|v| {
        comments.into_iter().for_each(|comment| {
            v.add_child(PaddedView::lrtb(
                comment.1 * 2,
                0,
                0,
                1,
                text_view::TextView::new(comment.0),
            ));
        })
    }))
    .on_event('q', move |s| match hn_client.get_top_stories() {
        Ok(stories) => {
            s.pop_layer();
            s.add_layer(story_view::get_story_view(stories, &hn_client))
        }
        Err(err) => {
            error!("failed to get top stories: {:#?}", err);
        }
    })
    .on_pre_event_inner('l', {
        let heights = heights.clone();
        move |s, _| {
            let id = s.get_focus_index();
            let (_, right) = heights.split_at(id + 1);
            let offset = right.iter().position(|&h| {
                debug!("h: {}", h);
                h <= heights[id]
            });
            let next_id = match offset {
                None => id,
                Some(offset) => id + offset + 1,
            };
            debug!("id: {}, next_id: {}", id, next_id);
            match s.set_focus_index(next_id) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        }
    })
    .on_pre_event_inner('h', {
        let heights = heights.clone();
        move |s, _| {
            let id = s.get_focus_index();
            let (left, _) = heights.split_at(id);
            let next_id = left.iter().rposition(|&h| h <= heights[id]).unwrap_or(id);
            match s.set_focus_index(next_id) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        }
    })
    .on_pre_event_inner('t', |s, _| {
        if s.len() > 0 {
            match s.set_focus_index(0) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        } else {
            Some(EventResult::Consumed(None))
        }
    })
    .on_pre_event_inner('b', |s, _| {
        if s.len() > 0 {
            match s.set_focus_index(s.len() - 1) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        } else {
            Some(EventResult::Consumed(None))
        }
    })
    .scrollable()
}
