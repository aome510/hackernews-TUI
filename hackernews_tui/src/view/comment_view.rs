use super::{article_view, async_view, help_view::HasHelpView, text_view, traits::*, utils};
use crate::prelude::*;
use crate::view::text_view::{StyledPaddingChar, TextPadding};

type CommentComponent = HideableView<PaddedView<text_view::TextView>>;

/// CommentView is a View displaying a list of comments in a HN story
pub struct CommentView {
    view: ScrollView<LinearLayout>,
    comments: Vec<client::HnText>,
    receiver: client::CommentReceiver,

    raw_command: String,
}

pub enum NavigationDirection {
    Next,
    Previous,
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: ScrollView<LinearLayout>);
}

impl CommentView {
    pub fn new(story_text: client::HnText, receiver: client::CommentReceiver) -> Self {
        let mut view = CommentView {
            view: LinearLayout::vertical()
                .child(HideableView::new(PaddedView::lrtb(
                    story_text.level * 2 + 1,
                    1,
                    0,
                    1,
                    text_view::TextView::new(story_text.text.clone()),
                )))
                .scrollable(),
            comments: vec![story_text],
            raw_command: String::new(),
            receiver,
        };

        view.try_update_comments();
        view
    }

    /// Check the comment receiver channel if there are new comments loaded
    /// then update the internal comment data accordingly.
    pub fn try_update_comments(&mut self) {
        let mut new_comments = vec![];
        // limit the number of top comments updated each time
        let mut limit = 5;
        while !self.receiver.is_empty() && limit > 0 {
            if let Ok(mut comments) = self.receiver.try_recv() {
                new_comments.append(&mut comments);
            }
            limit -= 1;
        }

        if new_comments.is_empty() {
            return;
        }

        new_comments.iter().for_each(|comment| {
            let text_view = text_view::TextView::new(comment.text.clone());
            self.add_item(HideableView::new(PaddedView::lrtb(
                comment.level * 2 + 1,
                1,
                0,
                1,
                if comment.level > 0 {
                    // get the padding style (color) based on the comment's height
                    //
                    // We use base 16 colors to display the comment's padding
                    let c = config::Color::from((comment.level % 16) as u8);
                    text_view
                        .padding(TextPadding::default().left(StyledPaddingChar::new('▎', c.into())))
                } else {
                    // add top padding for top comments, use the first color in the 16 base colors
                    let c = config::Color::from(0);
                    text_view
                        .padding(TextPadding::default().top(StyledPaddingChar::new('▔', c.into())))
                },
            )));
        });
        self.comments.append(&mut new_comments);

        // update the view's layout
        self.layout(
            self.get_inner_scroll_view()
                .get_scroller()
                .last_outer_size(),
        )
    }

    /// Return the id of the first comment (`direction` dependent)
    /// whose level is less than or equal `max_level`.
    pub fn find_comment_id_by_max_level(
        &self,
        start_id: usize,
        max_level: usize,
        direction: NavigationDirection,
    ) -> usize {
        match direction {
            NavigationDirection::Next => (start_id + 1..self.len())
                .find(|&id| self.comments[id].level <= max_level)
                .unwrap_or_else(|| self.len()),
            NavigationDirection::Previous => (0..start_id)
                .rfind(|&id| self.comments[id].level <= max_level)
                .unwrap_or(start_id),
        }
    }

    /// Return the id of the next visible comment (`direction` dependent)
    pub fn find_next_visible_comment(
        &self,
        start_id: usize,
        direction: NavigationDirection,
    ) -> usize {
        match direction {
            NavigationDirection::Next => (start_id + 1..self.len())
                .find(|&id| self.get_comment_component(id).is_visible())
                .unwrap_or_else(|| self.len()),
            NavigationDirection::Previous => (0..start_id)
                .rfind(|&id| self.get_comment_component(id).is_visible())
                .unwrap_or(start_id),
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

    /// Toggle the collapsing state of comments whose level is greater than the `min_level`.
    fn toggle_comment_collapse_state(&mut self, start_id: usize, min_level: usize) {
        // This function will be called recursively until it's unable to find any comments.
        //
        // **Note**: `PartiallyCollapsed` comment's state is unchanged, we only toggle its visibility.
        // Also, the state and visibility of such comment's children are unaffected as they should already
        // be in a collapsed state.
        if start_id == self.len() || self.comments[start_id].level <= min_level {
            return;
        }
        match self.comments[start_id].state {
            client::CollapseState::Collapsed => {
                self.comments[start_id].state = client::CollapseState::Normal;
                self.get_comment_component_mut(start_id).unhide();
                self.toggle_comment_collapse_state(start_id + 1, min_level)
            }
            client::CollapseState::Normal => {
                self.comments[start_id].state = client::CollapseState::Collapsed;
                self.get_comment_component_mut(start_id).hide();
                self.toggle_comment_collapse_state(start_id + 1, min_level)
            }
            client::CollapseState::PartiallyCollapsed => {
                let component = self.get_comment_component_mut(start_id);
                if component.is_visible() {
                    component.hide();
                } else {
                    component.unhide();
                }

                // skip toggling all child comments of the current comment
                let next_id = self.find_comment_id_by_max_level(
                    start_id,
                    self.comments[start_id].level,
                    NavigationDirection::Next,
                );
                self.toggle_comment_collapse_state(next_id, min_level)
            }
        };
    }

    /// Toggle the collapsing state of currently focused comment and its children
    pub fn toggle_collapse_focused_comment(&mut self) {
        let id = self.get_focus_index();
        let comment = self.comments[id].clone();
        match comment.state {
            client::CollapseState::Collapsed => {
                panic!(
                    "invalid collapse state `Collapsed` when calling `toggle_collapse_focused_comment`"
                );
            }
            client::CollapseState::PartiallyCollapsed => {
                self.get_comment_component_mut(id)
                    .get_inner_mut()
                    .get_inner_mut()
                    .set_content(comment.text);
                self.toggle_comment_collapse_state(id + 1, self.comments[id].level);
                self.comments[id].state = client::CollapseState::Normal;
            }
            client::CollapseState::Normal => {
                self.get_comment_component_mut(id)
                    .get_inner_mut()
                    .get_inner_mut()
                    .set_content(comment.minimized_text);
                self.toggle_comment_collapse_state(id + 1, self.comments[id].level);
                self.comments[id].state = client::CollapseState::PartiallyCollapsed;
            }
        };
    }

    inner_getters!(self.view: ScrollView<LinearLayout>);
}

impl ListViewContainer for CommentView {
    fn get_inner_list(&self) -> &LinearLayout {
        self.get_inner().get_inner()
    }

    fn get_inner_list_mut(&mut self) -> &mut LinearLayout {
        self.get_inner_mut().get_inner_mut()
    }

    fn on_set_focus_index(&mut self, old_id: usize, new_id: usize) {
        let direction = old_id <= new_id;

        // enable auto-scrolling when changing the focused index of the view
        self.scroll(direction);
    }
}

impl ScrollViewContainer for CommentView {
    type ScrollInner = LinearLayout;

    fn get_inner_scroll_view(&self) -> &ScrollView<LinearLayout> {
        self.get_inner()
    }

    fn get_inner_scroll_view_mut(&mut self) -> &mut ScrollView<LinearLayout> {
        self.get_inner_mut()
    }
}

fn construct_comment_main_view(
    story: &client::Story,
    receiver: client::CommentReceiver,
) -> impl View {
    let is_suffix_key = |c: &Event| -> bool {
        let comment_view_keymap = config::get_comment_view_keymap();
        comment_view_keymap.open_link_in_browser.has_event(c)
            || comment_view_keymap.open_link_in_article_view.has_event(c)
    };

    let comment_view_keymap = config::get_comment_view_keymap().clone();

    OnEventView::new(CommentView::new(story.text.clone(), receiver))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            s.try_update_comments();

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

            // don't allow the inner `LinearLayout` child view to handle the event
            // because of its pre-defined `on_event` function
            Some(EventResult::Ignored)
        })
        // comment navigation shortcuts
        .on_pre_event_inner(comment_view_keymap.prev_comment, |s, _| {
            s.set_focus_index(
                s.find_next_visible_comment(s.get_focus_index(), NavigationDirection::Previous),
            )
        })
        .on_pre_event_inner(comment_view_keymap.next_comment, |s, _| {
            let next_id =
                s.find_next_visible_comment(s.get_focus_index(), NavigationDirection::Next);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id =
                s.find_comment_id_by_max_level(id, s.comments[id].level, NavigationDirection::Next);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_level(
                id,
                s.comments[id].level,
                NavigationDirection::Previous,
            );
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_level(id, 0, NavigationDirection::Next);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_level(id, 0, NavigationDirection::Previous);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.parent_comment, move |s, _| {
            let id = s.get_focus_index();
            if s.comments[id].level > 0 {
                let next_id = s.find_comment_id_by_max_level(
                    id,
                    s.comments[id].level - 1,
                    NavigationDirection::Previous,
                );
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
                    utils::open_ith_link_in_browser(&s.comments[s.get_focus_index()].links, num)
                }
                Err(_) => None,
            }
        })
        .on_pre_event_inner(
            comment_view_keymap.open_link_in_article_view,
            |s, _| match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    utils::open_ith_link_in_article_view(
                        &s.comments[s.get_focus_index()].links,
                        num,
                    )
                }
                Err(_) => None,
            },
        )
        .on_pre_event_inner(comment_view_keymap.open_comment_in_browser, move |s, _| {
            let id = s.comments[s.get_focus_index()].id;
            let url = format!("{}/item?id={}", client::HN_HOST_URL, id);
            utils::open_url_in_browser(&url);
            Some(EventResult::Consumed(None))
        })
        // other commands
        .on_pre_event_inner(comment_view_keymap.toggle_collapse_comment, move |s, _| {
            s.toggle_collapse_focused_comment();
            Some(EventResult::Consumed(None))
        })
        .on_pre_event(comment_view_keymap.open_article_in_browser, {
            let url = story.get_url().into_owned();
            move |_| {
                utils::open_url_in_browser(&url);
            }
        })
        .on_pre_event(comment_view_keymap.open_article_in_article_view, {
            let url = story.url.clone();
            move |s| {
                if !url.is_empty() {
                    article_view::construct_and_add_new_article_view(s, &url)
                }
            }
        })
        .on_pre_event(comment_view_keymap.open_story_in_browser, {
            let url = story.story_url();
            move |_| {
                utils::open_url_in_browser(&url);
            }
        })
        .on_pre_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(CommentView::construct_on_event_help_view());
        })
        .on_scroll_events()
        .full_height()
}

/// Construct a comment view of a given story.
///
/// # Arguments:
/// * `story`: a Hacker News story
/// * `receiver`: a "subscriber" channel that gets comments asynchronously from another thread
pub fn construct_comment_view(
    story: &client::Story,
    receiver: client::CommentReceiver,
) -> impl View {
    let main_view = construct_comment_main_view(story, receiver);

    let mut view = LinearLayout::vertical()
        .child(utils::construct_view_title_bar(&format!(
            "Comment View - {}",
            story.title.source()
        )))
        .child(main_view)
        .child(utils::construct_footer_view::<CommentView>());
    view.set_focus_index(1)
        .unwrap_or(EventResult::Consumed(None));

    view
}

/// Retrieve comments of a story and construct a comment view of that story
pub fn construct_and_add_new_comment_view(
    s: &mut Cursive,
    client: &'static client::HNClient,
    story: &client::Story,
    pop_layer: bool,
) {
    let async_view = async_view::construct_comment_view_async(s, client, story);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
