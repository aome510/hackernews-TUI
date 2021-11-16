use super::help_view::HasHelpView;
use super::list_view::*;
use super::text_view;
use super::{article_view, async_view};
use crate::prelude::*;

type CommentComponent = HideableView<PaddedView<text_view::TextView>>;

/// CommentView is a View displaying a list of comments in a HN story
pub struct CommentView {
    view: ScrollListView,
    comments: Vec<client::Comment>,
    receiver: client::CommentReceiver,

    raw_command: String,
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: ScrollListView);
}

impl CommentView {
    /// Return a new CommentView given a comment list and the discussed story url
    pub fn new(receiver: client::CommentReceiver) -> Self {
        let mut view = CommentView {
            comments: vec![],
            view: LinearLayout::vertical().scrollable(),
            raw_command: String::new(),
            receiver,
        };
        view.try_update_comments();
        view
    }

    /// Check the `CommentReceiver` channel if there are new comments loaded
    /// then update the internal comment data accordingly.
    pub fn try_update_comments(&mut self) {
        let mut new_comments = vec![];
        while !self.receiver.is_empty() {
            if let Ok(mut comments) = self.receiver.try_recv() {
                new_comments.append(&mut comments);
            }
        }

        if new_comments.is_empty() {
            return;
        }

        new_comments.iter().for_each(|comment| {
            self.add_item(HideableView::new(PaddedView::lrtb(
                comment.height * 2,
                0,
                0,
                1,
                text_view::TextView::new(comment.text.clone()),
            )));
        });
        self.comments.append(&mut new_comments);

        self.layout(self.get_scroller().last_outer_size())
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
            client::CommentState::Collapsed => {
                self.comments[i].state = client::CommentState::Normal;
                self.get_comment_component_mut(i).unhide();
                self.toggle_comment_collapse_state(i + 1, min_height)
            }
            client::CommentState::Normal => {
                self.comments[i].state = client::CommentState::Collapsed;
                self.get_comment_component_mut(i).hide();
                self.toggle_comment_collapse_state(i + 1, min_height)
            }
            client::CommentState::PartiallyCollapsed => {
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
            client::CommentState::Collapsed => {
                panic!(
                    "invalid comment state `Collapsed` when calling `toggle_collapse_focused_comment`"
                );
            }
            client::CommentState::PartiallyCollapsed => {
                self.get_comment_component_mut(id)
                    .get_inner_mut()
                    .get_inner_mut()
                    .set_content(comment.text);
                self.toggle_comment_collapse_state(id + 1, self.comments[id].height);
                self.comments[id].state = client::CommentState::Normal;
            }
            client::CommentState::Normal => {
                self.get_comment_component_mut(id)
                    .get_inner_mut()
                    .get_inner_mut()
                    .set_content(comment.minimized_text);
                self.toggle_comment_collapse_state(id + 1, self.comments[id].height);
                self.comments[id].state = client::CommentState::PartiallyCollapsed;
            }
        };
    }

    inner_getters!(self.view: ScrollListView);
}

/// Return a main view of a CommentView displaying the comment list.
/// The main view of a CommentView is a View without status bar or footer.
fn get_comment_main_view(receiver: client::CommentReceiver) -> impl View {
    let comment_view_keymap = config::get_comment_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let comment_view_keymap = config::get_comment_view_keymap().clone();
        *c == comment_view_keymap.open_link_in_browser.into()
            || *c == comment_view_keymap.open_link_in_article_view.into()
    };

    OnEventView::new(CommentView::new(receiver))
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
            None
        })
        // scrolling shortcuts
        .on_pre_event_inner(comment_view_keymap.up, |s, _| {
            s.get_scroller_mut()
                .scroll_up(config::get_config().scroll_offset);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner(comment_view_keymap.down, |s, _| {
            s.get_scroller_mut()
                .scroll_down(config::get_config().scroll_offset);
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
                        utils::open_url_in_browser(&s.comments[id].links[num]);
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
            utils::open_url_in_browser(&url);
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
pub fn get_comment_view(story: &client::Story, receiver: client::CommentReceiver) -> impl View {
    let status_bar = utils::get_status_bar_with_desc(&format!("Comment View - {}", story.title));

    let main_view = get_comment_main_view(receiver);

    let mut view = LinearLayout::vertical()
        .child(status_bar)
        .child(main_view)
        .child(utils::construct_footer_view::<CommentView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    let id = story.id;

    OnEventView::new(view)
        .on_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(CommentView::construct_help_view());
        })
        .on_event(
            config::get_story_view_keymap()
                .open_article_in_browser
                .clone(),
            {
                let url = story.url.clone();
                move |_| {
                    utils::open_url_in_browser(&url);
                }
            },
        )
        .on_event(
            config::get_story_view_keymap()
                .open_article_in_article_view
                .clone(),
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
            config::get_story_view_keymap()
                .open_story_in_browser
                .clone(),
            move |_| {
                let url = format!("{}/item?id={}", client::HN_HOST_URL, id);
                utils::open_url_in_browser(&url);
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
