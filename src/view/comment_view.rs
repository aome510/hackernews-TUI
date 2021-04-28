use rayon::prelude::*;
use regex::Regex;
use std::thread;

use super::async_view;
use super::list_view::*;
use super::text_view;
use super::utils::*;

use crate::prelude::*;

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

    fn decode_html(s: &str) -> String {
        htmlescape::decode_html(s).unwrap_or(s.to_string())
    }

    /// Parse a comment in HTML text style to markdown text style (with colors)
    fn parse_single_comment(
        s: String,
        paragraph_re: &Regex,
        italic_re: &Regex,
        code_re: &Regex,
        link_re: &Regex,
    ) -> (StyledString, Vec<String>) {
        let mut s = paragraph_re.replace_all(&s, "${paragraph}\n").to_string();
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
                    let link = Self::decode_html(c.name("link").unwrap().as_str());

                    let range = m.range();
                    let mut prefix: String = s
                        .drain(std::ops::Range {
                            start: 0,
                            end: m.end(),
                        })
                        .collect();
                    prefix.drain(range);

                    if prefix.len() > 0 {
                        styled_s.append_plain(Self::decode_html(&prefix));
                    }

                    styled_s.append_styled(
                        format!("\"{}\" ", shorten_url(&link)),
                        Style::from(get_config_theme().link_text.color),
                    );
                    styled_s.append_styled(
                        format!("[{}]", links.len()),
                        ColorStyle::new(
                            PaletteColor::TitlePrimary,
                            get_config_theme().link_id_bg.color,
                        ),
                    );
                    links.push(link);
                    continue;
                }
            }
        }
        if s.len() > 0 {
            styled_s.append_plain(Self::decode_html(&s));
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
                    PaletteColor::Secondary,
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
    let comment_view_keymap = get_comment_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let comment_view_keymap = get_comment_view_keymap().clone();
        *c == comment_view_keymap.open_link_in_browser.into()
            || *c == comment_view_keymap.open_link_in_article_view.into()
    };

    construct_scroll_list_event_view(CommentView::new(story.clone(), comments, focus_id))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if '0' <= c && c <= '9' => {
                    s.raw_command.push(c);
                }
                _ => {
                    if !is_suffix_key(e) {
                        s.raw_command.clear();
                    }
                }
            };
            None
        })
        .on_pre_event_inner(comment_view_keymap.prev_comment, |s, _| {
            let id = s.get_focus_index();
            if id == 0 {
                None
            } else {
                s.set_focus_index(id - 1)
            }
        })
        .on_pre_event_inner(comment_view_keymap.next_comment, |s, _| {
            let id = s.get_focus_index();
            s.set_focus_index(id + 1)
        })
        .on_pre_event_inner(comment_view_keymap.next_leq_level_comment, move |s, _| {
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
        .on_pre_event_inner(comment_view_keymap.prev_leq_level_comment, move |s, _| {
            let heights = s.get_heights();
            let id = s.get_focus_index();
            let (left, _) = heights.split_at(id);
            let next_id = left.iter().rposition(|&h| h <= heights[id]).unwrap_or(id);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_top_level_comment, move |s, _| {
            let heights = s.get_heights();
            let id = s.get_focus_index();
            let (_, right) = heights.split_at(id + 1);
            let offset = right.iter().position(|&h| h == 0);
            let next_id = match offset {
                None => id,
                Some(offset) => id + offset + 1,
            };
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_top_level_comment, move |s, _| {
            let heights = s.get_heights();
            let id = s.get_focus_index();
            let (left, _) = heights.split_at(id);
            let next_id = left.iter().rposition(|&h| h == 0).unwrap_or(id);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.open_link_in_browser, |s, _| {
            match s.raw_command.parse::<usize>() {
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
            }
        })
        .on_pre_event_inner(
            comment_view_keymap.open_link_in_article_view,
            |s, _| match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    let id = s.get_focus_index();
                    if num < s.comments[id].links.len() {
                        let url = s.comments[id].links[num].clone();
                        Some(EventResult::with_cb({
                            move |s| article_view::add_article_view_layer(s, url.clone())
                        }))
                    } else {
                        Some(EventResult::Consumed(None))
                    }
                }
                Err(_) => None,
            },
        )
        .on_pre_event_inner(comment_view_keymap.reload_comment_view, move |s, _| {
            let focus_id = s.comments[s.get_focus_index()].id;
            Some(EventResult::with_cb({
                let client = client.clone();
                let story = s.story.clone();
                move |s| add_comment_view_layer(s, &client, &story, focus_id, true)
            }))
        })
        .on_pre_event_inner(comment_view_keymap.open_comment_in_browser, move |s, _| {
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
        .child(construct_footer_view::<CommentView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let id = story.id;

    OnEventView::new(view)
        .on_event(get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(CommentView::construct_help_view());
        })
        .on_event(get_story_view_keymap().open_article_in_browser.clone(), {
            {
                let url = story.url.clone();
                move |_| {
                    if url.len() > 0 {
                        let url = url.clone();
                        thread::spawn(move || {
                            if let Err(err) = webbrowser::open(&url) {
                                warn!("failed to open link {}: {}", url, err);
                            }
                        });
                    }
                }
            }
        })
        .on_event(
            get_story_view_keymap().open_article_in_article_view.clone(),
            {
                let url = story.url.clone();
                move |s| {
                    if url.len() > 0 {
                        article_view::add_article_view_layer(s, url.clone())
                    }
                }
            },
        )
        .on_event(
            get_story_view_keymap().open_story_in_browser.clone(),
            move |_| {
                thread::spawn(move || {
                    let url = format!("{}/item?id={}", hn_client::HN_HOST_URL, id);
                    if let Err(err) = webbrowser::open(&url) {
                        warn!("failed to open link {}: {}", url, err);
                    }
                });
            },
        )
}

/// Add a CommentView as a new layer to the main Cursive View
pub fn add_comment_view_layer(
    s: &mut Cursive,
    client: &hn_client::HNClient,
    story: &hn_client::Story,
    focus_id: u32,
    pop_layer: bool,
) {
    let async_view = async_view::get_comment_view_async(s, client, story, focus_id);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
