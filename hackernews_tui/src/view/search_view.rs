use super::help_view::*;
use super::text_view::EditableTextView;
use crate::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Copy, Clone)]
pub enum SearchViewMode {
    Navigation,
    Search,
}

struct Query {
    text_view: EditableTextView,
    needs_update: bool, // decide if the search view needs to be re-draw
}

impl Query {
    fn new() -> Self {
        Query {
            text_view: EditableTextView::new(),
            needs_update: false,
        }
    }
}

/// SearchView is a view displaying a search bar and
/// a matched story list matching a query string
pub struct SearchView {
    by_date: bool,
    page: usize,
    query: Arc<RwLock<Query>>,

    stories: Arc<RwLock<Vec<client::Story>>>,

    mode: SearchViewMode,

    view: LinearLayout,
    client: &'static client::HNClient,
    cb_sink: CbSink,
}

impl SearchView {
    fn get_matched_stories_view(&self, starting_id: usize) -> impl View {
        story_view::get_story_main_view(
            self.stories.read().unwrap().clone(),
            self.client,
            starting_id,
        )
        .full_height()
    }

    fn get_query_view(&self) -> impl View {
        let desc_view = TextView::new(StyledString::styled(
            format!(
                "Search (sort_by: {}): ",
                if self.by_date { "date" } else { "popularity" }
            ),
            ColorStyle::new(
                PaletteColor::TitlePrimary,
                get_config_theme().search_highlight_bg.color,
            ),
        ));

        let mut view = LinearLayout::horizontal().child(desc_view);
        match self.mode {
            SearchViewMode::Navigation => {
                view.add_child(TextView::new(
                    self.query.read().unwrap().text_view.get_text(),
                ));
            }
            SearchViewMode::Search => view.add_child(self.query.read().unwrap().text_view.clone()),
        };
        view
    }

    fn get_view(&self) -> LinearLayout {
        let starting_id = get_config().client.story_limit.search * self.page;
        let mut view = LinearLayout::vertical()
            .child(self.get_query_view())
            .child(self.get_matched_stories_view(starting_id));
        match self.mode {
            SearchViewMode::Search => {
                view.set_focus_index(0).unwrap();
            }
            SearchViewMode::Navigation => {
                view.set_focus_index(1).unwrap();
            }
        };
        view
    }

    fn update_view(&mut self) {
        if self.query.read().unwrap().needs_update {
            if self.stories.read().unwrap().is_empty() {
                self.mode = SearchViewMode::Search;
            };
            self.view = self.get_view();
            self.query.write().unwrap().needs_update = false;
        }
    }

    fn update_matched_stories(&mut self) {
        let self_stories = Arc::clone(&self.stories);
        let self_query = Arc::clone(&self.query);

        let cb_sink = self.cb_sink.clone();

        let client = self.client;
        let query = self.query.read().unwrap().text_view.get_text();
        let by_date = self.by_date;
        let page = self.page;

        // create a loading screen if the comand triggers
        // update_matched_stories is from NavigationMode's commands

        let is_navigation_mode = if let SearchViewMode::Navigation = self.mode {
            cb_sink
                .send(Box::new(|s| {
                    let loading_view = OnEventView::new(
                        Dialog::new()
                            .content(cursive_async_view::AsyncView::<TextView>::new(s, || {
                                cursive_async_view::AsyncState::Pending
                            }))
                            .max_width(32),
                    )
                    .on_event(EventTrigger::from_fn(|_| true), |_| {});
                    s.add_layer(loading_view);
                }))
                .unwrap();
            true
        } else {
            false
        };

        std::thread::spawn(
            move || match client.get_matched_stories(&query, by_date, page) {
                Err(err) => {
                    warn!(
                        "failed to get stories matching the query '{}': {:#?}",
                        query, err
                    );

                    // failed to get matched stories, but we still need
                    // to remove the loading dialog
                    if *self_query.read().unwrap().text_view.get_text() == query {
                        cb_sink
                            .send(Box::new(move |s| {
                                if is_navigation_mode {
                                    s.pop_layer();
                                }
                            }))
                            .unwrap();
                    }
                }
                Ok(stories) => {
                    // found matched stories...
                    // if the search query matches the current query,
                    // update stories, remove the loading dialog, and force redrawing the view
                    if *self_query.read().unwrap().text_view.get_text() == query {
                        (*self_stories.write().unwrap()) = stories;
                        self_query.write().unwrap().needs_update = true;

                        cb_sink
                            .send(Box::new(move |s| {
                                if is_navigation_mode {
                                    s.pop_layer();
                                }
                            }))
                            .unwrap();
                    }
                }
            },
        );
    }

    pub fn add_char(&mut self, c: char) -> Option<EventResult> {
        self.page = 0;
        self.query.write().unwrap().text_view.add_char(c);
        self.query.write().unwrap().needs_update = true;
        self.update_matched_stories();
        Some(EventResult::Consumed(None))
    }
    pub fn del_char(&mut self) -> Option<EventResult> {
        self.page = 0;
        self.query.write().unwrap().text_view.del_char();
        self.query.write().unwrap().needs_update = true;
        self.update_matched_stories();
        Some(EventResult::Consumed(None))
    }
    pub fn move_cursor_left(&mut self) -> Option<EventResult> {
        self.query.write().unwrap().text_view.move_cursor_left();
        self.query.write().unwrap().needs_update = true;
        Some(EventResult::Consumed(None))
    }
    pub fn move_cursor_right(&mut self) -> Option<EventResult> {
        self.query.write().unwrap().text_view.move_cursor_right();
        self.query.write().unwrap().needs_update = true;
        Some(EventResult::Consumed(None))
    }
    pub fn move_cursor_to_begin(&mut self) -> Option<EventResult> {
        self.query.write().unwrap().text_view.move_cursor_to_begin();
        self.query.write().unwrap().needs_update = true;
        Some(EventResult::Consumed(None))
    }
    pub fn move_cursor_to_end(&mut self) -> Option<EventResult> {
        self.query.write().unwrap().text_view.move_cursor_to_end();
        self.query.write().unwrap().needs_update = true;
        Some(EventResult::Consumed(None))
    }

    pub fn toggle_by_date(&mut self) -> Option<EventResult> {
        self.page = 0;
        self.by_date = !self.by_date;
        self.update_matched_stories();
        Some(EventResult::Consumed(None))
    }
    pub fn update_page(&mut self, next_page: bool) -> Option<EventResult> {
        if next_page {
            self.page += 1;
            self.update_matched_stories();
        } else if self.page > 0 {
            self.page -= 1;
            self.update_matched_stories();
        }
        Some(EventResult::Consumed(None))
    }

    pub fn new(client: &'static client::HNClient, cb_sink: CbSink) -> Self {
        let stories = Arc::new(RwLock::new(vec![]));
        let query = Arc::new(RwLock::new(Query::new()));
        let mut search_view = SearchView {
            by_date: false,
            page: 0,
            mode: SearchViewMode::Search,
            client,
            query,
            view: LinearLayout::vertical(),
            stories,
            cb_sink,
        };
        search_view.view = search_view.get_view();
        search_view
    }
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);

    fn wrap_required_size(&mut self, req: Vec2) -> Vec2 {
        self.update_view();
        self.view.required_size(req)
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.update_view();
        self.view.layout(size);
    }

    fn wrap_focus_view(&mut self, selector: &Selector<'_>) -> Result<(), ViewNotFound> {
        self.update_view();
        self.view.focus_view(selector)
    }

    fn wrap_take_focus(&mut self, _: Direction) -> bool {
        self.update_view();
        true
    }
}

/// Return a main view of a SearchView displaying the matched story list with a search bar.
/// The main view of a SearchView is a View without status bar or footer.
fn get_search_main_view(client: &'static client::HNClient, cb_sink: CbSink) -> impl View {
    let story_view_keymap = get_story_view_keymap().clone();
    let search_view_keymap = get_search_view_keymap().clone();
    let edit_keymap = get_edit_keymap().clone();

    OnEventView::new(SearchView::new(&client, cb_sink))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| match s.mode {
            SearchViewMode::Navigation => None,
            SearchViewMode::Search => match *e {
                Event::Char(c) => s.add_char(c),
                _ => None,
            },
        })
        .on_pre_event_inner(search_view_keymap.to_navigation_mode, |s, _| match s.mode {
            SearchViewMode::Navigation => None,
            SearchViewMode::Search => {
                s.mode = SearchViewMode::Navigation;
                s.view.set_focus_index(1).unwrap_or_else(|_| {});
                s.query.write().unwrap().needs_update = true;
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner(search_view_keymap.to_search_mode, |s, _| match s.mode {
            SearchViewMode::Search => None,
            SearchViewMode::Navigation => {
                s.mode = SearchViewMode::Search;
                s.view.set_focus_index(0).unwrap_or_else(|_| {});
                Some(EventResult::Consumed(None))
            }
        })
        // paging/filtering while in NavigationMode
        .on_pre_event_inner(story_view_keymap.toggle_sort_by, |s, _| match s.mode {
            SearchViewMode::Navigation => s.toggle_by_date(),
            SearchViewMode::Search => None,
        })
        .on_pre_event_inner(story_view_keymap.next_page, |s, _| match s.mode {
            SearchViewMode::Navigation => s.update_page(true),
            SearchViewMode::Search => None,
        })
        .on_pre_event_inner(story_view_keymap.prev_page, |s, _| match s.mode {
            SearchViewMode::Navigation => s.update_page(false),
            SearchViewMode::Search => None,
        })
        .on_pre_event_inner(edit_keymap.backward_delete_char, |s, _| s.del_char())
        .on_pre_event_inner(edit_keymap.move_cursor_left, |s, _| s.move_cursor_left())
        .on_pre_event_inner(edit_keymap.move_cursor_right, |s, _| s.move_cursor_right())
        .on_pre_event_inner(edit_keymap.move_cursor_to_begin, |s, _| {
            s.move_cursor_to_begin()
        })
        .on_pre_event_inner(edit_keymap.move_cursor_to_end, |s, _| {
            s.move_cursor_to_end()
        })
}

/// Return a view representing a SearchView that searches stories with queries
pub fn get_search_view(client: &'static client::HNClient, cb_sink: CbSink) -> impl View {
    let main_view = get_search_main_view(client, cb_sink);
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc("Search View"))
        .child(main_view)
        .child(construct_footer_view::<SearchView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(get_global_keymap().open_help_dialog.clone(), |s| {
        s.add_layer(SearchView::construct_help_view());
    })
}

/// Add a SearchView as a new layer to the main Cursive View
pub fn add_search_view_layer(s: &mut Cursive, client: &'static client::HNClient) {
    let cb_sink = s.cb_sink().clone();
    s.screen_mut()
        .add_transparent_layer(Layer::new(get_search_view(client, cb_sink)));
}
