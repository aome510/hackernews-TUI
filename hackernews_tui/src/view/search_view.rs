use super::{help_view::*, story_view, text_view::EditableTextView};
use crate::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq)]
enum SearchViewMode {
    Navigation,
    Search,
}

struct MatchedStories {
    pub query: String,
    pub page: usize,
    pub by_date: bool,
    pub stories: Vec<client::Story>,
}

/// SearchView is a View used to search stories
pub struct SearchView {
    mode: SearchViewMode,
    page: usize,
    by_date: bool,

    view: LinearLayout,

    sender: std::sync::mpsc::Sender<MatchedStories>,
    receiver: std::sync::mpsc::Receiver<MatchedStories>,

    client: &'static client::HNClient,
    cb_sink: CbSink,
}

impl SearchView {
    /// constructs new `SearchView`
    pub fn new(client: &'static client::HNClient, cb_sink: CbSink) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        let view = LinearLayout::vertical()
            .child(
                // construct a search bar view consisting of a description and an editable search text views
                LinearLayout::horizontal()
                    .child(TextView::new(StyledString::styled(
                        "Search: ",
                        config::get_config_theme().component_style.matched_highlight,
                    )))
                    .child(EditableTextView::new()),
            )
            .child(story_view::get_story_main_view(vec![], client, 0).full_height());

        Self {
            mode: SearchViewMode::Search,
            page: 0,
            by_date: false,
            view,
            client,
            cb_sink,
            sender,
            receiver,
        }
    }

    pub fn get_search_text_view_mut(&mut self) -> Option<&mut EditableTextView> {
        self.view
            .get_child_mut(0)?
            .downcast_mut::<LinearLayout>()?
            .get_child_mut(1)?
            .downcast_mut::<EditableTextView>()
    }

    /// retrieves stories matching the current query by making an external (API) request
    ///
    /// To ensure this function not blocking, message passing with channels is used.
    pub fn retrieve_matched_stories(&mut self) {
        let query = match self.get_search_text_view_mut() {
            None => return,
            Some(view) => view.get_text(),
        };
        let sender = self.sender.clone();
        let client = self.client.clone();
        let by_date = self.by_date;
        let page = self.page;

        // use a `cb_sink` to notify the `Cursive` renderer to re-draw the application
        // after successfully retrieving matched stories
        let cb_sink = self.cb_sink.clone();

        std::thread::spawn(
            move || match client.get_matched_stories(&query, by_date, page) {
                Ok(stories) => {
                    sender
                        .send(MatchedStories {
                            query,
                            stories,
                            by_date,
                            page,
                        })
                        .unwrap();
                    // send a dummy callback to `cb_sink`
                    cb_sink.send(Box::new(move |_| {})).unwrap();
                }
                Err(err) => {
                    warn!(
                        "failed to get matched stories (query={}, by_date={}, page={}): {}",
                        query, by_date, page, err
                    );
                }
            },
        );
    }

    /// tries to update the Story View representing matched stories based on
    /// the results from previous query requests
    pub fn try_update_view(&mut self) {
        let query = match self.get_search_text_view_mut() {
            None => return,
            Some(view) => view.get_text(),
        };
        while let Ok(matched_stories) = self.receiver.try_recv() {
            // got a `matched_stories` result but only care about the one matching current state
            if query == matched_stories.query
                && self.page == matched_stories.page
                && self.by_date == matched_stories.by_date
            {
                self.update_stories_view(matched_stories.stories);
            }
        }
    }

    /// updates the Story View with new matched stories
    fn update_stories_view(&mut self, stories: Vec<client::Story>) {
        self.view.remove_child(1);
        let starting_id = client::SEARCH_LIMIT * self.page;
        self.view.add_child(
            story_view::get_story_main_view(stories, self.client, starting_id).full_height(),
        );
        // the old Story View is deleted hence losing the current focus,
        // we need to place the focus back to the new Story View
        if self.mode == SearchViewMode::Navigation {
            self.view.set_focus_index(1).unwrap_or_else(|_| {
                // no Story View to focus on, or no stories to display,
                // change back to Search mode
                self.mode = SearchViewMode::Search;
                EventResult::Ignored
            });
        }
    }
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);

    fn wrap_layout(&mut self, size: Vec2) {
        self.try_update_view();
        self.view.layout(size);
    }
}

/// Return a main view of a SearchView displaying the matched story list with a search bar.
/// The main view of a SearchView is a View without status bar or footer.
fn get_search_main_view(client: &'static client::HNClient, cb_sink: CbSink) -> impl View {
    let story_view_keymap = config::get_story_view_keymap().clone();
    let search_view_keymap = config::get_search_view_keymap().clone();

    OnEventView::new(SearchView::new(client, cb_sink))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| match s.mode {
            SearchViewMode::Navigation => None,
            SearchViewMode::Search => {
                let view = s.get_search_text_view_mut()?;
                match *e {
                    Event::Char(c) => {
                        view.add_char(c);
                        s.page = 0;
                        s.retrieve_matched_stories();
                    }
                    _ => {
                        // handle editing shortcuts when in the search mode
                        let edit_keymap = config::get_edit_keymap().clone();
                        if edit_keymap.backward_delete_char.has_event(e) {
                            view.del_char();
                            s.page = 0;
                            s.retrieve_matched_stories();
                        } else if edit_keymap.move_cursor_left.has_event(e) {
                            view.move_cursor_left();
                        } else if edit_keymap.move_cursor_right.has_event(e) {
                            view.move_cursor_right();
                        } else if edit_keymap.move_cursor_to_begin.has_event(e) {
                            view.move_cursor_to_begin();
                        } else if edit_keymap.move_cursor_to_end.has_event(e) {
                            view.move_cursor_to_end();
                        } else {
                            return Some(EventResult::Ignored);
                        }
                    }
                }
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner(search_view_keymap.to_navigation_mode, |s, _| match s.mode {
            SearchViewMode::Navigation => None,
            SearchViewMode::Search => {
                if s.view.set_focus_index(1).is_ok() {
                    s.mode = SearchViewMode::Navigation;
                }
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner(search_view_keymap.to_search_mode, |s, _| match s.mode {
            SearchViewMode::Search => None,
            SearchViewMode::Navigation => {
                if s.view.set_focus_index(0).is_ok() {
                    s.mode = SearchViewMode::Search;
                }
                Some(EventResult::Consumed(None))
            }
        })
        // paging/filtering while in NavigationMode
        .on_pre_event_inner(story_view_keymap.toggle_sort_by_date, |s, _| match s.mode {
            SearchViewMode::Navigation => {
                s.page = 0;
                s.by_date = !s.by_date;
                s.retrieve_matched_stories();
                Some(EventResult::Consumed(None))
            }
            SearchViewMode::Search => Some(EventResult::Ignored),
        })
        .on_pre_event_inner(story_view_keymap.next_page, |s, _| match s.mode {
            SearchViewMode::Navigation => {
                s.page += 1;
                s.retrieve_matched_stories();
                Some(EventResult::Consumed(None))
            }
            SearchViewMode::Search => Some(EventResult::Ignored),
        })
        .on_pre_event_inner(story_view_keymap.prev_page, |s, _| match s.mode {
            SearchViewMode::Navigation => {
                if s.page > 0 {
                    s.page -= 1;
                    s.retrieve_matched_stories();
                }
                Some(EventResult::Consumed(None))
            }
            SearchViewMode::Search => Some(EventResult::Ignored),
        })
}

/// Return a view representing a SearchView that searches stories with queries
pub fn get_search_view(client: &'static client::HNClient, cb_sink: CbSink) -> impl View {
    let main_view = get_search_main_view(client, cb_sink);
    let mut view = LinearLayout::vertical()
        .child(utils::construct_view_title_bar("Search View"))
        .child(main_view)
        .child(utils::construct_footer_view::<SearchView>());
    view.set_focus_index(1)
        .unwrap_or(EventResult::Consumed(None));

    OnEventView::new(view).on_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
        s.add_layer(SearchView::construct_on_event_help_view());
    })
}

/// Add a SearchView as a new layer to the main Cursive View
pub fn add_search_view_layer(s: &mut Cursive, client: &'static client::HNClient) {
    let cb_sink = s.cb_sink().clone();
    s.screen_mut()
        .add_transparent_layer(Layer::new(get_search_view(client, cb_sink)));
}
