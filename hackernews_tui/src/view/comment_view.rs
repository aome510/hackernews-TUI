use super::{article_view, async_view, help_view::HasHelpView, text_view, traits::*, utils};
use crate::prelude::*;
use crate::view::text_view::{StyledPaddingChar, TextPadding};

type SingleItemView = HideableView<PaddedView<text_view::TextView>>;

/// CommentView is a View displaying a list of comments in a HN story
pub struct CommentView {
    view: ScrollView<LinearLayout>,
    items: Vec<HnItem>,
    data: PageData,

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
    pub fn new(data: PageData) -> Self {
        let mut view = CommentView {
            view: LinearLayout::vertical()
                .child(HideableView::new(PaddedView::lrtb(
                    1,
                    1,
                    0,
                    1,
                    text_view::TextView::new(
                        data.root_item.text(
                            data.vote_state
                                .get(&data.root_item.id.to_string())
                                .map(|v| v.upvoted),
                        ),
                    ),
                )))
                .scrollable(),
            items: vec![data.root_item.clone()],
            raw_command: String::new(),
            data,
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
        while !self.data.comment_receiver.is_empty() && limit > 0 {
            if let Ok(mut comments) = self.data.comment_receiver.try_recv() {
                new_comments.append(&mut comments);
            }
            limit -= 1;
        }

        if new_comments.is_empty() {
            return;
        }

        let mut new_items = new_comments
            .into_iter()
            .map(Into::<HnItem>::into)
            .collect::<Vec<_>>();

        new_items.iter().for_each(|item| {
            let text_view = text_view::TextView::new(item.text(self.get_vote_status(item.id)));
            self.add_item(HideableView::new(PaddedView::lrtb(
                item.level * 2 + 1,
                1,
                0,
                1,
                if item.level > 0 {
                    // get the padding style (color) based on the comment's height
                    //
                    // We use base 16 colors to display the comment's padding
                    let c = config::Color::from((item.level % 16) as u8);
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
        self.items.append(&mut new_items);

        // update the view's layout
        self.layout(
            self.get_inner_scroll_view()
                .get_scroller()
                .last_outer_size(),
        )
    }

    /// Return the id of the first item (`direction` dependent),
    /// whose level is less than or equal `max_level`.
    pub fn find_item_id_by_max_level(
        &self,
        start_id: usize,
        max_level: usize,
        direction: NavigationDirection,
    ) -> usize {
        match direction {
            NavigationDirection::Next => (start_id + 1..self.len())
                .find(|&id| self.items[id].level <= max_level)
                .unwrap_or_else(|| self.len()),
            NavigationDirection::Previous => (0..start_id)
                .rfind(|&id| self.items[id].level <= max_level)
                .unwrap_or(start_id),
        }
    }

    /// Return the id of the next visible item (`direction` dependent)
    pub fn find_next_visible_item(&self, start_id: usize, direction: NavigationDirection) -> usize {
        match direction {
            NavigationDirection::Next => (start_id + 1..self.len())
                .find(|&id| self.get_item_view(id).is_visible())
                .unwrap_or_else(|| self.len()),
            NavigationDirection::Previous => (0..start_id)
                .rfind(|&id| self.get_item_view(id).is_visible())
                .unwrap_or(start_id),
        }
    }

    fn get_vote_status(&self, item_id: u32) -> Option<bool> {
        self.data
            .vote_state
            .get(&item_id.to_string())
            .map(|v| v.upvoted)
    }

    fn get_item_view(&self, id: usize) -> &SingleItemView {
        self.get_item(id)
            .unwrap()
            .downcast_ref::<SingleItemView>()
            .unwrap()
    }

    fn get_item_view_mut(&mut self, id: usize) -> &mut SingleItemView {
        self.get_item_mut(id)
            .unwrap()
            .downcast_mut::<SingleItemView>()
            .unwrap()
    }

    /// Toggle the collapsing state of items whose levels are greater than the `min_level`.
    fn toggle_items_collapse_state(&mut self, start_id: usize, min_level: usize) {
        // This function will be called recursively until it's unable to find any items.
        //
        // Note: collapsed item's state is unchanged, we only toggle its visibility.
        // Also, the state and visibility of such item's children are unaffected as they should already
        // be in a hidden state (as result of that item's collapsed state).
        if start_id == self.len() || self.items[start_id].level <= min_level {
            return;
        }
        match self.items[start_id].display_state {
            DisplayState::Hidden => {
                self.items[start_id].display_state = DisplayState::Normal;
                self.get_item_view_mut(start_id).unhide();
                self.toggle_items_collapse_state(start_id + 1, min_level)
            }
            DisplayState::Normal => {
                self.items[start_id].display_state = DisplayState::Hidden;
                self.get_item_view_mut(start_id).hide();
                self.toggle_items_collapse_state(start_id + 1, min_level)
            }
            DisplayState::Minimized => {
                let component = self.get_item_view_mut(start_id);
                if component.is_visible() {
                    component.hide();
                } else {
                    component.unhide();
                }

                // skip toggling all children of the current item
                let next_id = self.find_item_id_by_max_level(
                    start_id,
                    self.items[start_id].level,
                    NavigationDirection::Next,
                );
                self.toggle_items_collapse_state(next_id, min_level)
            }
        };
    }

    /// Toggle the collapsing state of currently focused item and its children
    pub fn toggle_collapse_focused_item(&mut self) {
        let id = self.get_focus_index();
        match self.items[id].display_state {
            DisplayState::Hidden => {
                panic!(
                    "invalid collapse state `Collapsed` when calling `toggle_collapse_focused_item`"
                );
            }
            DisplayState::Minimized => {
                self.toggle_items_collapse_state(id + 1, self.items[id].level);
                self.items[id].display_state = DisplayState::Normal;
            }
            DisplayState::Normal => {
                self.toggle_items_collapse_state(id + 1, self.items[id].level);
                self.items[id].display_state = DisplayState::Minimized;
            }
        };
        self.update_item_text_content(id);
    }

    /// Update the `id`-th item's text content based on its state-based text
    pub fn update_item_text_content(&mut self, id: usize) {
        let new_content = self.items[id].text(self.get_vote_status(self.items[id].id));
        self.get_item_view_mut(id)
            .get_inner_mut()
            .get_inner_mut()
            .set_content(new_content);
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

fn construct_comment_main_view(client: &'static client::HNClient, data: PageData) -> impl View {
    let is_suffix_key = |c: &Event| -> bool {
        let comment_view_keymap = config::get_comment_view_keymap();
        comment_view_keymap.open_link_in_browser.has_event(c)
            || comment_view_keymap.open_link_in_article_view.has_event(c)
    };

    let comment_view_keymap = config::get_comment_view_keymap().clone();

    let article_url = data.url.clone();
    let page_url = format!("{}/item?id={}", client::HN_HOST_URL, data.root_item.id);

    OnEventView::new(CommentView::new(data))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), move |s, e| {
            s.try_update_comments();

            match *e {
                Event::Char(c) if c.is_ascii_digit() => {
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
        .on_pre_event_inner(comment_view_keymap.vote, |s, _| {
            let id = s.get_focus_index();
            let item = &s.items[id];
            if let Some(VoteData { auth, upvoted }) =
                s.data.vote_state.get_mut(&item.id.to_string())
            {
                std::thread::spawn({
                    let id = item.id;
                    let upvoted = *upvoted;
                    let auth = auth.clone();
                    let client = client.clone();
                    move || {
                        if let Err(err) = client.vote(id, &auth, upvoted) {
                            tracing::error!("Failed to vote HN item (id={id}): {err}");
                        }
                    }
                });

                // assume the vote request always succeeds because we don't want users
                // to feel a delay as a result of the request's latency when voting.
                *upvoted = !(*upvoted);
                s.update_item_text_content(id);
            }
            Some(EventResult::Consumed(None))
        })
        // comment navigation shortcuts
        .on_pre_event_inner(comment_view_keymap.prev_comment, |s, _| {
            s.set_focus_index(
                s.find_next_visible_item(s.get_focus_index(), NavigationDirection::Previous),
            )
        })
        .on_pre_event_inner(comment_view_keymap.next_comment, |s, _| {
            let next_id = s.find_next_visible_item(s.get_focus_index(), NavigationDirection::Next);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id =
                s.find_item_id_by_max_level(id, s.items[id].level, NavigationDirection::Next);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_leq_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id =
                s.find_item_id_by_max_level(id, s.items[id].level, NavigationDirection::Previous);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.next_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_item_id_by_max_level(id, 0, NavigationDirection::Next);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.prev_top_level_comment, move |s, _| {
            let id = s.get_focus_index();
            let next_id = s.find_item_id_by_max_level(id, 0, NavigationDirection::Previous);
            s.set_focus_index(next_id)
        })
        .on_pre_event_inner(comment_view_keymap.parent_comment, move |s, _| {
            let id = s.get_focus_index();
            if s.items[id].level > 0 {
                let next_id = s.find_item_id_by_max_level(
                    id,
                    s.items[id].level - 1,
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
                    utils::open_ith_link_in_browser(&s.items[s.get_focus_index()].links, num)
                }
                Err(_) => None,
            }
        })
        .on_pre_event_inner(
            comment_view_keymap.open_link_in_article_view,
            move |s, _| match s.raw_command.parse::<usize>() {
                Ok(num) => {
                    s.raw_command.clear();
                    utils::open_ith_link_in_article_view(
                        client,
                        &s.items[s.get_focus_index()].links,
                        num,
                    )
                }
                Err(_) => None,
            },
        )
        .on_pre_event_inner(comment_view_keymap.open_comment_in_browser, move |s, _| {
            let id = s.items[s.get_focus_index()].id;
            let url = format!("{}/item?id={}", client::HN_HOST_URL, id);
            utils::open_url_in_browser(&url);
            Some(EventResult::Consumed(None))
        })
        // other commands
        .on_pre_event_inner(comment_view_keymap.toggle_collapse_comment, move |s, _| {
            s.toggle_collapse_focused_item();
            Some(EventResult::Consumed(None))
        })
        .on_pre_event(comment_view_keymap.open_article_in_browser, {
            let url = article_url.clone();
            move |_| {
                utils::open_url_in_browser(&url);
            }
        })
        .on_pre_event(comment_view_keymap.open_article_in_article_view, {
            let url = article_url;
            move |s| {
                if !url.is_empty() {
                    article_view::construct_and_add_new_article_view(client, s, &url)
                }
            }
        })
        .on_pre_event(comment_view_keymap.open_story_in_browser, {
            let url = page_url;
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

pub fn construct_comment_view(client: &'static client::HNClient, data: PageData) -> impl View {
    let title = format!("Comment View - {}", data.title,);
    let main_view = construct_comment_main_view(client, data);

    let mut view = LinearLayout::vertical()
        .child(utils::construct_view_title_bar(&title))
        .child(main_view)
        .child(utils::construct_footer_view::<CommentView>());
    view.set_focus_index(1)
        .unwrap_or(EventResult::Consumed(None));

    view
}

/// Retrieve comments in a Hacker News item and construct a comment view of that item
pub fn construct_and_add_new_comment_view(
    s: &mut Cursive,
    client: &'static client::HNClient,
    item_id: u32,
    pop_layer: bool,
) {
    let async_view = async_view::construct_comment_view_async(s, client, item_id);
    if pop_layer {
        s.pop_layer();
    }
    s.screen_mut().add_transparent_layer(Layer::new(async_view));
}
