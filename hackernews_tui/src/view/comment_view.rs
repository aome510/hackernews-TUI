use super::help_view::HasHelpView;
use super::text_view;
use super::traits::*;
use super::{article_view, async_view};
use crate::prelude::*;
use crate::view::text_view::StyledPaddingChar;
use crate::view::text_view::TextPadding;

type CommentComponent = HideableView<PaddedView<text_view::TextView>>;

/// CommentView is a View displaying a list of comments in a HN story
pub struct CommentView {
    view: ScrollView<LinearLayout>,
    comments: Vec<client::HnText>,
    receiver: client::CommentReceiver,

    raw_command: String,
}

impl ViewWrapper for CommentView {
    wrap_impl!(self.view: ScrollView<LinearLayout>);
}

impl CommentView {
    /// Return a new CommentView given a comment list and the discussed story url
    pub fn new(main_text: client::HnText, receiver: client::CommentReceiver) -> Self {
        let mut view = CommentView {
            view: LinearLayout::vertical()
                .child(HideableView::new(PaddedView::lrtb(
                    main_text.level * 2 + 1,
                    1,
                    0,
                    1,
                    text_view::TextView::new(main_text.text.clone()),
                )))
                .scrollable(),
            comments: vec![main_text],
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

        // TODO: handle this
        // self.layout(self.get_scroller().last_outer_size())
    }

    /// Return the id of the first comment (`direction` dependent)
    /// whose level is less than or equal `max_level`.
    pub fn find_comment_id_by_max_level(
        &self,
        start_id: usize,
        max_level: usize,
        direction: bool,
    ) -> usize {
        if direction {
            // ->
            (start_id + 1..self.len())
                .find(|&id| self.comments[id].level <= max_level)
                .unwrap_or_else(|| self.len())
        } else {
            // <-
            (0..start_id)
                .rfind(|&id| self.comments[id].level <= max_level)
                .unwrap_or(start_id)
        }
    }

    /// Return the id of the next visible comment
    pub fn find_next_visible_comment(&self, start_id: usize, direction: bool) -> usize {
        if direction {
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
        if i == self.len() || self.comments[i].level <= min_height {
            return;
        }
        match self.comments[i].state {
            client::CollapseState::Collapsed => {
                self.comments[i].state = client::CollapseState::Normal;
                self.get_comment_component_mut(i).unhide();
                self.toggle_comment_collapse_state(i + 1, min_height)
            }
            client::CollapseState::Normal => {
                self.comments[i].state = client::CollapseState::Collapsed;
                self.get_comment_component_mut(i).hide();
                self.toggle_comment_collapse_state(i + 1, min_height)
            }
            client::CollapseState::PartiallyCollapsed => {
                let component = self.get_comment_component_mut(i);
                if component.is_visible() {
                    component.hide();
                } else {
                    component.unhide();
                }

                // skip toggling all child comments of the current comment
                let next_id = self.find_comment_id_by_max_level(i, self.comments[i].level, true);
                self.toggle_comment_collapse_state(next_id, min_height)
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
                    "invalid comment state `Collapsed` when calling `toggle_collapse_focused_comment`"
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
}

impl ScrollContainer for CommentView {
    fn get_inner_scroller(&self) -> &scroll::Core {
        self.get_inner().get_scroller()
    }

    fn get_inner_scroller_mut(&mut self) -> &mut scroll::Core {
        self.get_inner_mut().get_scroller_mut()
    }
}

/// Return a main view of a CommentView displaying the comment list.
/// The main view of a CommentView is a View without status bar or footer.
fn get_comment_main_view(
    main_text: client::HnText,
    receiver: client::CommentReceiver,
) -> impl View {
    let comment_view_keymap = config::get_comment_view_keymap().clone();

    let is_suffix_key = |c: &Event| -> bool {
        let comment_view_keymap = config::get_comment_view_keymap().clone();
        comment_view_keymap.open_link_in_browser.has_event(c)
            || comment_view_keymap.open_link_in_article_view.has_event(c)
    };

    OnEventView::new(CommentView::new(main_text, receiver))
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
            let next_id = s.find_comment_id_by_max_level(id, s.comments[id].level, true);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_level(id, s.comments[id].level, false);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_level(id, 0, true);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_comment_id_by_max_level(id, 0, false);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.parent_comment, move |s, _| {
            let id = s.get_focus_index();
            if s.comments[id].level > 0 {
                let next_id = s.find_comment_id_by_max_level(id, s.comments[id].level - 1, false);
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
                    if num > 0 && num <= s.comments[id].links.len() {
                        utils::open_url_in_browser(&s.comments[id].links[num - 1]);
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
                    if num > 0 && num <= s.comments[id].links.len() {
                        let url = s.comments[id].links[num - 1].clone();
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
        .on_scroll_events()
        .full_height()
}

/// Return a CommentView given a comment list and the discussed story's url/title
pub fn get_comment_view(story: &client::Story, receiver: client::CommentReceiver) -> impl View {
    let status_bar =
        utils::construct_view_title_bar(&format!("Comment View - {}", story.title.source()));

    let main_view = get_comment_main_view(story.text.clone(), receiver);

    let mut view = LinearLayout::vertical()
        .child(status_bar)
        .child(main_view)
        .child(utils::construct_footer_view::<CommentView>());
    view.set_focus_index(1)
        .unwrap_or(EventResult::Consumed(None));

    let id = story.id;

    OnEventView::new(view)
        .on_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
            s.add_layer(CommentView::construct_on_event_help_view());
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
