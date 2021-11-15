use std::sync::Arc;

use super::async_view;
use super::list_view::*;
use super::text_view;
use crate::prelude::*;
use parking_lot::RwLock;
use regex::Regex;

type CommentComponent = HideableView<PaddedView<text_view::TextView>>;

#[derive(Debug, Clone)]
/// CommentState represents the state of a single comment component
enum CommentState {
    Collapsed,
    PartiallyCollapsed,
    Normal,
}

#[derive(Debug, Clone)]
pub struct Comment {
    state: CommentState,
    top_comment_id: u32,
    id: u32,

    text: StyledString,
    minimized_text: StyledString,

    height: usize,
    links: Vec<String>,
}

impl Comment {
    pub fn new(
        top_comment_id: u32,
        id: u32,
        text: StyledString,
        minimized_text: StyledString,
        height: usize,
        links: Vec<String>,
    ) -> Self {
        Comment {
            state: CommentState::Normal,
            top_comment_id,
            id,
            text,
            minimized_text,
            height,
            links,
        }
    }
}

/// CommentView is a View displaying a list of comments in a HN story
pub struct CommentView {
    view: ScrollListView,
    comments: Arc<RwLock<Vec<Comment>>>,

    raw_command: String,
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: ScrollListView);

    fn wrap_layout(&mut self, size: Vec2) {
        // to support focus the last focused comment on reloading,
        // scroll to the focus element during the view initialization
        let is_init = self.get_inner().get_scroller().last_available_size() == Vec2::zero();

        self.with_view_mut(|v| v.layout(size));

        if is_init {
            self.scroll(true)
        }
    }
}

impl CommentView {
    /// Return a new CommentView given a comment list and the discussed story url
    pub fn new(receiver: crossbeam_channel::Receiver<client::Comment>) -> Self {
        let comment_view = CommentView {
            comments: Arc::new(RwLock::new(vec![])),
            view: LinearLayout::vertical().scrollable(),
            raw_command: String::new(),
        };
        comment_view
    }

    fn decode_html(s: &str) -> String {
        htmlescape::decode_html(s).unwrap_or_else(|_| s.to_string())
    }

    /// Parse a comment in HTML text style to markdown text style (with colors)
    fn parse_single_comment(
        s: &str,
        paragraph_re: &Regex,
        italic_re: &Regex,
        code_re: &Regex,
        link_re: &Regex,
    ) -> (StyledString, Vec<String>) {
        let mut s = paragraph_re.replace_all(s, "${paragraph}\n\n").to_string();
        if s.ends_with("\n\n") {
            s.remove(s.len() - 1);
        }

        s = italic_re.replace_all(&s, "*${text}*").to_string();
        s = code_re.replace_all(&s, "```\n${code}\n```").to_string();

        let mut links: Vec<String> = vec![];
        let mut styled_s = StyledString::new();
        // replace the `<a href="${link}">...</a>` pattern one-by-one with "${link}".
        // cannot use `replace_all` because we want to replace a match string with a `StyledString`
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

                    if !prefix.is_empty() {
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
        if !s.is_empty() {
            styled_s.append_plain(Self::decode_html(&s));
        }
        (styled_s, links)
    }

    /// Parse comments recursively into readable texts with styles and colors
    fn parse_comments(
        comments: &[client::Comment],
        height: usize,
        top_comment_id: u32,
    ) -> Vec<Comment> {
        let paragraph_re = Regex::new(r"<p>(?s)(?P<paragraph>.*?)</p>").unwrap();
        let italic_re = Regex::new(r"<i>(?s)(?P<text>.+?)</i>").unwrap();
        let code_re = Regex::new(r"<pre><code>(?s)(?P<code>.+?)[\n]*</code></pre>").unwrap();
        let link_re = Regex::new(r#"<a\s+?href="(?P<link>.+?)"(?s).+?</a>"#).unwrap();

        comments
            .iter()
            .flat_map(|comment| {
                let top_comment_id = if height == 0 {
                    comment.id
                } else {
                    top_comment_id
                };
                let mut subcomments =
                    Self::parse_comments(&comment.children, height + 1, top_comment_id);
                let mut comment_string = StyledString::styled(
                    format!(
                        "{} {} ago\n",
                        comment.author,
                        get_elapsed_time_as_text(comment.time),
                    ),
                    PaletteColor::Secondary,
                );

                let (comment_content, links) = Self::parse_single_comment(
                    &comment.text,
                    &paragraph_re,
                    &italic_re,
                    &code_re,
                    &link_re,
                );
                comment_string.append(comment_content);

                // minimized_comment is used to display collapsed comment
                let minimized_comment_string = StyledString::styled(
                    format!(
                        "{} {} ago ({} more)",
                        comment.author,
                        get_elapsed_time_as_text(comment.time),
                        subcomments.len() + 1,
                    ),
                    PaletteColor::Secondary,
                );

                subcomments.insert(
                    0,
                    Comment::new(
                        top_comment_id,
                        comment.id,
                        comment_string,
                        minimized_comment_string,
                        height,
                        links,
                    ),
                );
                subcomments
            })
            .collect()
    }

    /// Return the `id` of the first (`direction` dependent and starting but not including `start_id`)
    /// comment which has the `height` less than or equal the `max_height`
    pub fn find_comment_id_by_max_height(
        &self,
        start_id: usize,
        max_height: usize,
        direction: bool,
    ) -> usize {
        if direction {
            // ->
            (start_id + 1..self.len())
                .find(|&id| self.comments[id].height <= max_height)
                .unwrap_or_else(|| self.len())
        } else {
            // <-
            (0..start_id)
                .rfind(|&id| self.comments[id].height <= max_height)
                .unwrap_or(start_id)
        }
    }

    /// Return the id of the next visible comment
    pub fn find_next_visible_comment(&self, start_id: usize, go_left: bool) -> usize {
        if go_left {
            // ->
            (start_id + 1..self.len())
                .find(|&id| self.get_comment_component(id).is_visible())
                .unwrap_or_else(|| self.len())
        } else {
            // <-
            (0..start_id)
                .rfind(|&id| self.get_comment_component(id).is_visible())
                .unwrap_or(start_id)
        }
    }

    fn get_comment_component(&self, id: usize) -> &CommentComponent {
        self.get_item(id)
            .unwrap()
            .downcast_ref::<CommentComponent>()
            .unwrap()
    }

    fn get_comment_component_mut(&mut self, id: usize) -> &mut CommentComponent {
        self.get_item_mut(id)
            .unwrap()
            .downcast_mut::<CommentComponent>()
            .unwrap()
    }

    /// Toggle the collapsing state of comments whose height is greater
    /// than the `min_height`.
    /// **Note** `PartiallyCollapsed` comment's state is unchanged, only toggle its visibility.
    /// Also, the state and visibility of such comment's children are unaffected.
    fn toggle_comment_collapse_state(&mut self, i: usize, min_height: usize) {
        if i == self.len() || self.comments[i].height <= min_height {
            return;
        }
        match self.comments[i].state {
            CommentState::Collapsed => {
                self.comments[i].state = CommentState::Normal;
                self.get_comment_component_mut(i).unhide();
                self.toggle_comment_collapse_state(i + 1, min_height)
            }
            CommentState::Normal => {
                self.comments[i].state = CommentState::Collapsed;
                self.get_comment_component_mut(i).hide();
                self.toggle_comment_collapse_state(i + 1, min_height)
            }
            CommentState::PartiallyCollapsed => {
                let component = self.get_comment_component_mut(i);
                if component.is_visible() {
                    component.hide();
                } else {
                    component.unhide();
                }

                // skip toggling all child comments of the current comment
                let next_id = self.find_comment_id_by_max_height(i, self.comments[i].height, true);
                self.toggle_comment_collapse_state(next_id, min_height)
            }
        };
    }

    /// Toggle the collapsing state of currently focused comment and its children
    pub fn toggle_collapse_focused_comment(&mut self) {
        let id = self.get_focus_index();
        let comment = self.comments[id].clone();
        match comment.state {
            CommentState::Collapsed => {
                panic!(
                    "invalid comment state `Collapsed` when calling `toggle_collapse_focused_comment`"
                );
            }
            CommentState::PartiallyCollapsed => {
                self.get_comment_component_mut(id)
                    .get_inner_mut()
                    .get_inner_mut()
                    .set_content(comment.text);
                self.toggle_comment_collapse_state(id + 1, self.comments[id].height);
                self.comments[id].state = CommentState::Normal;
            }
            CommentState::Normal => {
                self.get_comment_component_mut(id)
                    .get_inner_mut()
                    .get_inner_mut()
                    .set_content(comment.minimized_text);
                self.toggle_comment_collapse_state(id + 1, self.comments[id].height);
                self.comments[id].state = CommentState::PartiallyCollapsed;
            }
        };
    }

    inner_getters!(self.view: ScrollListView);
}

/// Return a main view of a CommentView displaying the comment list.
/// The main view of a CommentView is a View without status bar or footer.
fn get_comment_main_view(receiver: crossbeam_channel::Receiver<Comment>) -> impl View {
    let comment_view_keymap = get_comment_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let comment_view_keymap = get_comment_view_keymap().clone();
        *c == comment_view_keymap.open_link_in_browser.into()
            || *c == comment_view_keymap.open_link_in_article_view.into()
    };

    OnEventView::new(CommentView::new(receiver))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            match *e {
                Event::Char(c) if ('0'..='9').contains(&c) => {
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
        // scrolling shortcuts
        .on_pre_event_inner(comment_view_keymap.up, |s, _| {
            s.get_scroller_mut().scroll_up(get_config().scroll_offset);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(comment_view_keymap.down, |s, _| {
            s.get_scroller_mut().scroll_down(get_config().scroll_offset);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(comment_view_keymap.page_up, |s, _| {
            let height = s.get_scroller_mut().last_available_size().y;
            s.get_scroller_mut().scroll_up(height / 2);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(comment_view_keymap.page_down, |s, _| {
            let height = s.get_scroller_mut().last_available_size().y;
            s.get_scroller_mut().scroll_down(height / 2);
            Some(EventResult::Consumed(None))
        })
        // comment navigation shortcuts
        .on_pre_event_inner(comment_view_keymap.prev_comment, |s, _| {
            s.set_focus_index(s.find_next_visible_comment(s.get_focus_index(), false))
        })
        .on_pre_event_inner(comment_view_keymap.next_comment, |s, _| {
            let next_id = s.find_next_visible_comment(s.get_focus_index(), true);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_height(id, s.comments[id].height, true);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_height(id, s.comments[id].height, false);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_height(id, 0, true);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_height(id, 0, false);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.parent_comment, move |s, _| {
            let id = s.get_focus_index();
            if s.comments[id].height > 0 {
                let next_id = s.find_comment_id_by_max_height(id, s.comments[id].height - 1, false);
                s.set_focus_index(next_id)
            } else {
                Some(EventResult::Consumed(None))
            }
        })
        // open external link shortcuts
        .on_pre_event_inner(comment_view_keymap.open_link_in_browser, |s, _| {
            match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    let id = s.get_focus_index();
                    if num < s.comments[id].links.len() {
                        open_url_in_browser(&s.comments[id].links[num]);
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
                            move |s| article_view::add_article_view_layer(s, &url)
                        }))
                    } else {
                        Some(EventResult::Consumed(None))
                    }
                }
                Err(_) => None,
            },
        )
        .on_pre_event_inner(comment_view_keymap.open_comment_in_browser, move |s, _| {
            let id = s.comments[s.get_focus_index()].id;
            let url = format!("{}/item?id={}", client::HN_HOST_URL, id);
            open_url_in_browser(&url);
            Some(EventResult::Consumed(None))
        })
        // other commands
        .on_pre_event_inner(comment_view_keymap.toggle_collapse_comment, move |s, _| {
            s.toggle_collapse_focused_comment();
            Some(EventResult::Consumed(None))
        })
        .full_height()
}

/// Return a CommentView given a comment list and the discussed story's url/title
pub fn get_comment_view(
    story: &client::Story,
    receiver: crossbeam_channel::Receiver<Comment>,
) -> impl View {
    let match_re = Regex::new(r"<em>(?P<match>.*?)</em>").unwrap();
    let story_title = match_re.replace_all(&story.title, "${match}");

    let status_bar = get_status_bar_with_desc(&format!("Comment View - {}", story_title));

    let main_view = get_comment_main_view(receiver);

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
            let url = story.url.clone();
            move |_| {
                open_url_in_browser(&url);
            }
        })
        .on_event(
            get_story_view_keymap().open_article_in_article_view.clone(),
            {
                let url = story.url.clone();
                move |s| {
                    if !url.is_empty() {
                        article_view::add_article_view_layer(s, &url)
                    }
                }
            },
        )
        .on_event(
            get_story_view_keymap().open_story_in_browser.clone(),
            move |_| {
                let url = format!("{}/item?id={}", client::HN_HOST_URL, id);
                open_url_in_browser(&url);
            },
        )
}

/// Add a CommentView as a new layer to the main Cursive View
pub fn add_comment_view_layer(
    s: &mut Cursive,
    client: &'static client::HNClient,
    story: &client::Story,
    pop_layer: bool,
) {
    let async_view = async_view::get_comment_view_async(s, client, story);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
