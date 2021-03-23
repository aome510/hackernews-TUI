use super::event_view;
use super::help_view::*;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;
use std::thread;

#[derive(Debug, Clone)]
pub struct Story {
    pub id: u32,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Comment {
    id: u32,
    text: StyledString,
    height: usize,
    links: Vec<String>,
}

impl Story {
    pub fn new(story: &hn_client::Story) -> Self {
        Story {
            id: story.id,
            title: story.title.clone(),
            url: story.url.clone(),
        }
    }
}

impl Comment {
    pub fn new(id: u32, text: StyledString, height: usize, links: Vec<String>) -> Self {
        Comment {
            id,
            text,
            height,
            links,
        }
    }
}

/// CommentView is a View displaying a list of comments in a HN story
pub struct CommentView {
    story_metadata: Story,
    view: LinearLayout,
    comments: Vec<Comment>,

    raw_command: String,
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
                    format!("\"{}\"", shorten_url(link)),
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

/// Parse comments recursively into readable texts with styles and colors
fn parse_comment_text_list(comments: &Vec<hn_client::Comment>, height: usize) -> Vec<Comment> {
    let paragraph_re = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
    let italic_re = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
    let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
    let link_re = Regex::new(r#"<a\s+?href="(?P<link>.+?)".+?</a>"#).unwrap();

    comments
        .par_iter()
        .flat_map(|comment| {
            let mut subcomments = parse_comment_text_list(&comment.children, height + 1);
            let mut comment_string = StyledString::styled(
                format!(
                    "{} {} ago\n",
                    comment.author,
                    get_elapsed_time_as_text(comment.time),
                ),
                DESC_COLOR,
            );

            let (comment_content, links) = parse_raw_comment(
                comment.text.clone(),
                &paragraph_re,
                &italic_re,
                &code_re,
                &link_re,
            );
            comment_string.append(comment_content);

            subcomments.insert(0, Comment::new(comment.id, comment_string, height, links));
            subcomments
        })
        .collect()
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: LinearLayout);
}

impl CommentView {
    /// Return a new CommentView given a comment list and the discussed story url
    pub fn new(story_metadata: Story, comments: &Vec<hn_client::Comment>) -> Self {
        let comments = parse_comment_text_list(comments, 0);
        let view = LinearLayout::vertical().with(|v| {
            comments.iter().for_each(|comment| {
                v.add_child(PaddedView::lrtb(
                    comment.height * 2,
                    0,
                    0,
                    1,
                    text_view::TextView::new(comment.text.clone()),
                ));
            })
        });
        CommentView {
            story_metadata,
            view,
            comments,
            raw_command: String::new(),
        }
    }

    /// Get the height of each comment in the comment tree
    pub fn get_heights(&self) -> Vec<usize> {
        self.comments.iter().map(|comment| comment.height).collect()
    }

    crate::raw_command!();

    inner_getters!(self.view: LinearLayout);
}

/// Return a main view of a CommentView displaying the comment list.
/// The main view of a CommentView is a View without status bar or footer.
fn get_comment_main_view(story_metadata: Story, comments: &Vec<hn_client::Comment>) -> impl View {
    event_view::construct_list_event_view(CommentView::new(story_metadata, comments))
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
                Err(_) => None,
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
                Err(_) => None,
            }
        })
        .on_pre_event_inner('f', |s, _| match s.get_raw_command_as_number() {
            Ok(num) => {
                s.clear_raw_command();
                let id = s.get_inner().get_focus_index();
                if num < s.comments[id].links.len() {
                    let url = s.comments[id].links[num].clone();
                    thread::spawn(move || {
                        if let Err(err) = webbrowser::open(&url) {
                            warn!("failed to open link {}: {}", url, err);
                        }
                    });
                    Some(EventResult::Consumed(None))
                } else {
                    Some(EventResult::Consumed(None))
                }
            }
            Err(_) => None,
        })
        .on_pre_event_inner('O', move |s, _| {
            if s.story_metadata.url.len() > 0 {
                let url = s.story_metadata.url.clone();
                thread::spawn(move || {
                    if let Err(err) = webbrowser::open(&url) {
                        warn!("failed to open link {}: {}", url, err);
                    }
                });
                Some(EventResult::Consumed(None))
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('S', move |s, _| {
            let id = s.story_metadata.id;
            thread::spawn(move || {
                let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                if let Err(err) = webbrowser::open(&url) {
                    warn!("failed to open link {}: {}", url, err);
                }
            });
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('C', move |s, _| {
            let id = s.comments[s.get_inner().get_focus_index()].id;
            thread::spawn(move || {
                let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                if let Err(err) = webbrowser::open(&url) {
                    warn!("failed to open link {}: {}", url, err);
                }
            });
            Some(EventResult::Consumed(None))
        })
        .full_height()
        .scrollable()
}

/// Return a CommentView given a comment list and the discussed story's url/title
pub fn get_comment_view(
    story_metadata: Story,
    comments: &Vec<hn_client::Comment>,
    client: &hn_client::HNClient,
) -> impl View {
    let match_re = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();
    let story_title = match_re.replace_all(&story_metadata.title, "${match}");
    let status_bar = get_status_bar_with_desc(&format!("Comment View - {}", story_title));

    let main_view = get_comment_main_view(story_metadata, comments);

    let mut view = LinearLayout::vertical()
        .child(status_bar)
        .child(main_view)
        .child(construct_footer_view::<CommentView>(client));
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('h') | Event::AltChar('h') => true,
            _ => false,
        }),
        |s| {
            s.add_layer(CommentView::construct_help_view());
        },
    )
}
