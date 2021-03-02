use super::event_view;
use super::text_view;
use super::theme::*;
use crate::prelude::*;

pub struct CommentView {
    view: LinearLayout,
    comments: Vec<(StyledString, usize, Vec<String>)>,
}

/// Parse a raw comment in HTML text to markdown text (with colors)
fn parse_raw_comment(
    s: String,
    paragraph_re: &Regex,
    italic_re: &Regex,
    code_re: &Regex,
    link_re: &Regex,
) -> (StyledString, Vec<String>) {
    let mut s = htmlescape::decode_html(&s).unwrap_or(s);
    s = paragraph_re.replace_all(&s, "${paragraph}\n").to_string();
    s = italic_re.replace_all(&s, "*${text}*").to_string();
    s = code_re.replace_all(&s, "```\n${code}\n```").to_string();
    let mut links: Vec<String> = vec![];
    let mut styled_s = StyledString::new();
    loop {
        match link_re.captures(&s.clone()) {
            None => break,
            Some(c) => {
                let m = c.get(0).unwrap();
                let link = c.name("link").unwrap().as_str();

                let range = m.range();
                let mut prefix: String = s
                    .drain(std::ops::Range {
                        start: 0,
                        end: m.end(),
                    })
                    .collect();
                prefix.drain(range);

                if prefix.len() > 0 {
                    styled_s.append_plain(&prefix);
                }

                styled_s.append_styled(link, Style::from(LINK_COLOR));
                styled_s.append_styled(
                    links.len().to_string(),
                    ColorStyle::new(LINK_ID_FRONT, LINK_ID_BACK),
                );
                links.push(link.to_string());
                continue;
            }
        }
    }
    if s.len() > 0 {
        styled_s.append_plain(&s)
    }
    (styled_s, links)
}

/// Retrieve all comments recursively and parse them into readable texts with styles and colors
fn parse_comment_text_list(
    comments: &Vec<Box<hn_client::Comment>>,
    height: usize,
) -> Vec<(StyledString, usize, Vec<String>)> {
    let paragraph_re = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
    let italic_re = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
    let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
    let link_re = Regex::new(r#"<a\s+?href=(?P<link>".+?").+?</a>"#).unwrap();

    comments
        .par_iter()
        .flat_map(|comment| {
            let comment = &comment.as_ref();
            let mut subcomments = parse_comment_text_list(&comment.children, height + 1);
            let mut comment_string = StyledString::plain(format!(
                "{} {} ago\n",
                comment.author.clone().unwrap_or("[deleted]".to_string()),
                super::get_elapsed_time_as_text(comment.time),
            ));

            let (comment_content, links) = parse_raw_comment(
                comment.text.clone().unwrap_or("[deleted]".to_string()),
                &paragraph_re,
                &italic_re,
                &code_re,
                &link_re,
            );
            comment_string.append(comment_content);

            subcomments.insert(0, (comment_string, height, links));
            subcomments
        })
        .collect()
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: LinearLayout);
}

impl CommentView {
    /// Return a new CommentView based on the list of comments received from HN Client
    pub fn new(comments: &Vec<Box<hn_client::Comment>>) -> Self {
        let comments = parse_comment_text_list(&comments, 0);
        let view = LinearLayout::vertical().with(|v| {
            comments.iter().for_each(|comment| {
                v.add_child(PaddedView::lrtb(
                    comment.1 * 2,
                    0,
                    0,
                    1,
                    text_view::TextView::new(comment.0.clone()),
                ));
            })
        });
        CommentView { view, comments }
    }

    /// Get the height of each comment in the comment tree
    pub fn get_heights(&self) -> Vec<usize> {
        self.comments.iter().map(|comment| comment.1).collect()
    }

    inner_getters!(self.view: LinearLayout);
}

/// Return a cursive's View representing a CommentView with
/// registered event handlers and scrollable trait.
pub fn get_comment_view(
    hn_client: &hn_client::HNClient,
    comments: &Vec<Box<hn_client::Comment>>,
) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();

    event_view::construct_event_view(CommentView::new(comments))
        .on_event('q', move |s| match hn_client.get_top_stories() {
            Ok(stories) => {
                s.pop_layer();
                s.add_layer(story_view::get_story_view(stories, &hn_client))
            }
            Err(err) => {
                error!("failed to get top stories: {:#?}", err);
            }
        })
        .on_pre_event_inner('l', move |s, _| {
            let heights = s.get_heights();
            let s = s.get_inner_mut();
            let id = s.get_focus_index();
            let (_, right) = heights.split_at(id + 1);
            let offset = right.iter().position(|&h| h <= heights[id]);
            let next_id = match offset {
                None => id,
                Some(offset) => id + offset + 1,
            };
            match s.set_focus_index(next_id) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        })
        .on_pre_event_inner('h', move |s, _| {
            let heights = s.get_heights();
            let s = s.get_inner_mut();
            let id = s.get_focus_index();
            let (left, _) = heights.split_at(id);
            let next_id = left.iter().rposition(|&h| h <= heights[id]).unwrap_or(id);
            match s.set_focus_index(next_id) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => Some(EventResult::Ignored),
            }
        })
        .on_pre_event_inner('t', |s, _| {
            let s = s.get_inner_mut();
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
            let s = s.get_inner_mut();
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
