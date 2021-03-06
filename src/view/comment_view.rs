use super::event_view;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;

/// CommentView is a View displaying a comment thread of a HN story
pub struct CommentView {
    story_url: Option<String>,
    raw_command: String,
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
    // replace the <a href="${link}">...</a> pattern one-by-one with "${link}".
    // cannot use replace_all as above because we want to color links and link ids
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

                styled_s.append_styled(
                    format!("\"{}\"", shorten_url(link.to_string())),
                    Style::from(LINK_COLOR),
                );
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
    let link_re = Regex::new(r#"<a\s+?href="(?P<link>.+?)".+?</a>"#).unwrap();

    comments
        .par_iter()
        .flat_map(|comment| {
            let comment = &comment.as_ref();
            let mut subcomments = parse_comment_text_list(&comment.children, height + 1);
            let mut comment_string = StyledString::styled(
                format!(
                    "{} {} ago\n",
                    comment.author.clone().unwrap_or("[deleted]".to_string()),
                    get_elapsed_time_as_text(comment.time),
                ),
                DESC_COLOR,
            );

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
    pub fn new(
        story_text: StyledString,
        story_url: Option<String>,
        comments: &Vec<Box<hn_client::Comment>>,
    ) -> Self {
        let comments = parse_comment_text_list(&comments, 0);
        let view = LinearLayout::vertical()
            .child(PaddedView::lrtb(
                0,
                0,
                0,
                2,
                text_view::TextView::new(story_text),
            ))
            .with(|v| {
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
        CommentView {
            story_url,
            raw_command: "".to_string(),
            view,
            comments,
        }
    }

    /// Get the height of each comment in the comment tree
    pub fn get_heights(&self) -> Vec<usize> {
        self.comments.iter().map(|comment| comment.1).collect()
    }

    crate::raw_command!();

    inner_getters!(self.view: LinearLayout);
}

/// Return a cursive's View representing a CommentView with
/// registered event handlers and scrollable trait.
pub fn get_comment_view(
    story_text: StyledString,
    story_url: Option<String>,
    client: &hn_client::HNClient,
    comments: &Vec<Box<hn_client::Comment>>,
) -> impl View {
    let client = client.clone();

    event_view::construct_event_view(CommentView::new(story_text, story_url, comments))
        .on_event('q', move |s| {
            s.pop_layer();
            let async_view = async_view::get_story_view_async(s, &client);
            s.add_layer(async_view);
        })
        .on_pre_event_inner('l', move |s, _| {
            let heights = s.get_heights();
            let s = s.get_inner_mut();
            let id = s.get_focus_index();
            if id == 0 {
                return None;
            }
            let id = id - 1;
            let (_, right) = heights.split_at(id + 1);
            let offset = right.iter().position(|&h| h <= heights[id]);
            let next_id = match offset {
                None => id,
                Some(offset) => id + offset + 1,
            };
            match s.set_focus_index(next_id + 1) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => None,
            }
        })
        .on_pre_event_inner('h', move |s, _| {
            let heights = s.get_heights();
            let s = s.get_inner_mut();
            let id = s.get_focus_index();
            if id == 0 {
                return None;
            }
            let id = id - 1;
            let (left, _) = heights.split_at(id);
            let next_id = left.iter().rposition(|&h| h <= heights[id]).unwrap_or(id);
            match s.set_focus_index(next_id + 1) {
                Ok(_) => Some(EventResult::Consumed(None)),
                Err(_) => None,
            }
        })
        .on_pre_event_inner('t', |s, _| {
            let s = s.get_inner_mut();
            if s.len() > 0 {
                match s.set_focus_index(0) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(_) => None,
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
                    Err(_) => None,
                }
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('f', |s, _| match s.get_raw_command_as_number() {
            Ok(num) => {
                s.clear_raw_command();
                let id = s.get_inner().get_focus_index();
                if id == 0 {
                    return None;
                }
                let links = s.comments[id - 1].2.clone();
                if num < links.len() {
                    match webbrowser::open(&links[num]) {
                        Ok(_) => Some(EventResult::Consumed(None)),
                        Err(err) => {
                            warn!("failed to open link {}: {}", links[num], err);
                            None
                        }
                    }
                } else {
                    Some(EventResult::Consumed(None))
                }
            }
            Err(_) => None,
        })
        .on_pre_event_inner('O', move |s, _| {
            if s.story_url.is_some() {
                let url = s.story_url.clone().unwrap();
                match webbrowser::open(&url) {
                    Ok(_) => Some(EventResult::Consumed(None)),
                    Err(err) => {
                        warn!("failed to open link {}: {}", url, err);
                        None
                    }
                }
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        .scrollable()
}
