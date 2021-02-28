use super::event_view;
use super::text_view;
use super::theme::*;
use crate::prelude::*;

pub struct CommentView {
    comments: Vec<Box<hn_client::Comment>>,
    links: Vec<String>,
}

impl CommentView {
    pub fn new(comments: Vec<Box<hn_client::Comment>>) -> Self {
        CommentView {
            comments,
            links: vec![],
        }
    }

    /// Parse a raw text from HN API to human-readable string
    fn format_hn_text(
        &self,
        s: String,
        paragraph_re: &Regex,
        italic_re: &Regex,
        code_re: &Regex,
        link_re: &Regex,
    ) -> StyledString {
        let mut s = htmlescape::decode_html(&s).unwrap_or(s);
        s = paragraph_re.replace_all(&s, "${paragraph}\n").to_string();
        s = italic_re.replace_all(&s, "*${text}*").to_string();
        s = code_re.replace_all(&s, "```\n${code}\n```").to_string();
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
                    continue;
                }
            }
        }
        if s.len() > 0 {
            styled_s.append_plain(&s)
        }
        styled_s
    }

    /// Retrieve all comments recursively and parse them into readable texts with styles and colors
    fn parse_comment_text_list(
        &self,
        comments: &Vec<Box<hn_client::Comment>>,
        height: usize,
    ) -> Vec<(StyledString, usize)> {
        let paragraph_re = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
        let italic_re = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
        let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
        let link_re = Regex::new(r#"<a\s+?href=(?P<link>".+?").+?</a>"#).unwrap();

        comments
            .par_iter()
            .flat_map(|comment| {
                let comment = &comment.as_ref();
                let mut subcomments = self.parse_comment_text_list(&comment.children, height + 1);
                let mut comment_string = StyledString::plain(format!(
                    "{} {} ago\n",
                    comment.author.clone().unwrap_or("[deleted]".to_string()),
                    super::get_elapsed_time_as_text(comment.time),
                ));
                comment_string.append(self.format_hn_text(
                    comment.text.clone().unwrap_or("[deleted]".to_string()),
                    &paragraph_re,
                    &italic_re,
                    &code_re,
                    &link_re,
                ));
                subcomments.insert(0, (comment_string, height));
                subcomments
            })
            .collect()
    }

    /// Return a cursive's View from a comment list
    pub fn get_comment_view(&self, hn_client: &hn_client::HNClient) -> impl IntoBoxedView {
        let hn_client = hn_client.clone();

        let comments = self.parse_comment_text_list(&self.comments, 0);
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
                let offset = right.iter().position(|&h| h <= heights[id]);
                let next_id = match offset {
                    None => id,
                    Some(offset) => id + offset + 1,
                };
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
}
