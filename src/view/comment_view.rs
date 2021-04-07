use cursive::view::scroll::Scroller;

use super::async_view;
use super::help_view::*;
use super::list_view::*;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;
use std::thread;

#[derive(Debug, Clone)]
pub struct Comment {
    id: u32,
    text: StyledString,
    height: usize,
    links: Vec<String>,
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
    story: hn_client::Story,
    view: ScrollListView,
    comments: Vec<Comment>,

    raw_command: String,
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: ScrollListView);

    fn wrap_layout(&mut self, size: Vec2) {
        // to support focus the last focused comment on reloading,
        // scroll the the focus element on view initialization
        let is_init = self.get_inner().get_scroller().last_available_size() == Vec2::zero();

        self.with_view_mut(|v| v.layout(size));

        if is_init {
            self.scroll(true)
        }
    }
}

impl CommentView {
    /// Return a new CommentView given a comment list and the discussed story url
    pub fn new(story: hn_client::Story, comments: &Vec<hn_client::Comment>, focus_id: u32) -> Self {
        let comments = Self::parse_comments(comments, 0);
        let mut view = LinearLayout::vertical().with(|v| {
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
        if let Some(focus_id) = comments.iter().position(|comment| comment.id == focus_id) {
            view.set_focus_index(focus_id).unwrap();
        }
        CommentView {
            story,
            comments,
            view: view.scrollable(),
            raw_command: String::new(),
        }
    }

    /// Parse a comment in HTML text style to markdown text style (with colors)
    fn parse_single_comment(
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
        // cannot use replace_all as above because we want to replace the matched pattern
        // by a StyledString with specific colors.
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
    fn parse_comments(comments: &Vec<hn_client::Comment>, height: usize) -> Vec<Comment> {
        let paragraph_re = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
        let italic_re = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
        let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
        let link_re = Regex::new(r#"<a\s+?href="(?P<link>.+?)".+?</a>"#).unwrap();

        comments
            .par_iter()
            .flat_map(|comment| {
                let mut subcomments = Self::parse_comments(&comment.children, height + 1);
                let mut comment_string = StyledString::styled(
                    format!(
                        "{} {} ago\n",
                        comment.author,
                        get_elapsed_time_as_text(comment.time),
                    ),
                    DESC_COLOR,
                );

                let (comment_content, links) = Self::parse_single_comment(
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

    /// Get the height of each comment in the comment tree
    pub fn get_heights(&self) -> Vec<usize> {
        self.comments.iter().map(|comment| comment.height).collect()
    }

    inner_getters!(self.view: ScrollListView);
}

/// Return a main view of a CommentView displaying the comment list.
/// The main view of a CommentView is a View without status bar or footer.
fn get_comment_main_view(
    story: &hn_client::Story,
    comments: &Vec<hn_client::Comment>,
    client: &hn_client::HNClient,
    focus_id: u32,
) -> impl View {
    let client = client.clone();

    construct_scroll_list_event_view(CommentView::new(story.clone(), comments, focus_id))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| {
            match *e {
                Event::Char(c) if '0' <= c && c <= '9' => {
                    s.raw_command.push(c);
                }
                Event::Char('f') => {}
                _ => {
                    s.raw_command.clear();
                }
            };
            None
        })
        .on_pre_event_inner('l', move |s, _| {
            let heights = s.get_heights();
            let id = s.get_focus_index();
            let (_, right) = heights.split_at(id + 1);
            let offset = right.iter().position(|&h| h <= heights[id]);
            let next_id = match offset {
                None => id,
                Some(offset) => id + offset + 1,
            };
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner('h', move |s, _| {
            let heights = s.get_heights();
            let id = s.get_focus_index();
            let (left, _) = heights.split_at(id);
            let next_id = left.iter().rposition(|&h| h <= heights[id]).unwrap_or(id);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner('f', |s, _| match s.raw_command.parse::<usize>() {
            Ok(num) => {
                s.raw_command.clear();
                let id = s.get_focus_index();
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
        .on_pre_event_inner('r', move |s, _| {
            let focus_id = s.comments[s.get_focus_index()].id;
            Some(EventResult::with_cb({
                let client = client.clone();
                let story = s.story.clone();
                move |s| {
                    let async_view =
                        async_view::get_comment_view_async(s, &client, &story, focus_id);
                    s.pop_layer();
                    s.screen_mut().add_transparent_layer(Layer::new(async_view))
                }
            }))
        })
        .on_pre_event_inner('C', move |s, _| {
            let id = s.comments[s.get_focus_index()].id;
            thread::spawn(move || {
                let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                if let Err(err) = webbrowser::open(&url) {
                    warn!("failed to open link {}: {}", url, err);
                }
            });
            Some(EventResult::Consumed(None))
        })
        .full_height()
}

/// Return a CommentView given a comment list and the discussed story's url/title
pub fn get_comment_view(
    story: &hn_client::Story,
    comments: &Vec<hn_client::Comment>,
    client: &hn_client::HNClient,
    focus_id: u32,
) -> impl View {
    let match_re = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();
    let story_title = match_re.replace_all(&story.title, "${match}");
    let status_bar = get_status_bar_with_desc(&format!("Comment View - {}", story_title));

    let main_view = get_comment_main_view(story, comments, &client, focus_id);

    let mut view = LinearLayout::vertical()
        .child(status_bar)
        .child(main_view)
        .child(construct_footer_view::<CommentView>(client));
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let id = story.id;
    let url = story.url.clone();

    OnEventView::new(view)
        .on_event(
            EventTrigger::from_fn(|e| match e {
                Event::CtrlChar('h') | Event::AltChar('h') => true,
                _ => false,
            }),
            |s| {
                s.add_layer(CommentView::construct_help_view());
            },
        )
        .on_event('O', move |_| {
            if url.len() > 0 {
                let url = url.clone();
                thread::spawn(move || {
                    if let Err(err) = webbrowser::open(&url) {
                        warn!("failed to open link {}: {}", url, err);
                    }
                });
            }
        })
        .on_event('S', move |_| {
            thread::spawn(move || {
                let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                if let Err(err) = webbrowser::open(&url) {
                    warn!("failed to open link {}: {}", url, err);
                }
            });
        })
}
