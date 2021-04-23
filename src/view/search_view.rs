use std::{
    sync::{Arc, RwLock},
    thread,
};

use cursive_async_view::{AsyncState, AsyncView};

use super::help_view::*;
use super::text_view;
use super::utils::*;

use crate::prelude::*;

#[derive(Clone)]
pub enum SearchViewMode {
    Navigation,
    Search,
}

/// SearchView is a view displaying a search bar and
/// a matched story list matching a query string
pub struct SearchView {
    by_date: bool,
    page: usize,
    // ("query_text", "need_update_view") pair
    query: Arc<RwLock<(String, bool)>>,

    stories: Arc<RwLock<Vec<hn_client::Story>>>,

    mode: SearchViewMode,

    view: LinearLayout,
    client: hn_client::HNClient,
    cb_sink: CbSink,
}

impl SearchView {
    fn get_matched_stories_view(
        stories: Vec<hn_client::Story>,
        client: &hn_client::HNClient,
        starting_id: usize,
    ) -> impl View {
        let client = client.clone();
        story_view::get_story_main_view(stories, &client, starting_id).full_height()
    }

    fn get_query_text_view(query: String, by_date: bool) -> impl View {
        let mut style_string = StyledString::styled(
            format!(
                "Search (sort_by: {}):",
                if by_date { "date" } else { "popularity" }
            ),
            ColorStyle::new(
                PaletteColor::TitlePrimary,
                get_config_theme().search_highlight_bg.color,
            ),
        );
        style_string.append_plain(format!(" {}", query));
        text_view::TextView::new(style_string)
            .fixed_height(1)
            .full_width()
    }

    fn get_search_view(
        mode: SearchViewMode,
        query: &str,
        by_date: bool,
        page: usize,
        stories: Vec<hn_client::Story>,
        client: &hn_client::HNClient,
    ) -> LinearLayout {
        let starting_id = CONFIG.get().unwrap().client.story_limit.search * page;
        let mut view = LinearLayout::vertical()
            .child(Self::get_query_text_view(query.to_string(), by_date))
            .child(Self::get_matched_stories_view(stories, client, starting_id));
        match mode {
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
        if self.query.read().unwrap().1 {
            let stories = self.stories.read().unwrap().clone();
            if stories.len() == 0 {
                self.mode = SearchViewMode::Search;
            };

            self.view = Self::get_search_view(
                self.mode.clone(),
                &self.query.read().unwrap().0.clone(),
                self.by_date,
                self.page,
                stories,
                &self.client,
            );
            self.query.write().unwrap().1 = false;
        }
    }

    fn update_matched_stories(&mut self) {
        let self_stories = Arc::clone(&self.stories);
        let self_query = Arc::clone(&self.query);

        let cb_sink = self.cb_sink.clone();

        let client = self.client.clone();
        let query = self.query.read().unwrap().0.clone();
        let by_date = self.by_date;
        let page = self.page;

        let mut is_navigation_mode = false;
        if let SearchViewMode::Navigation = self.mode {
            is_navigation_mode = true;
            cb_sink
                .send(Box::new(|s| {
                    let loading_view = Dialog::new()
                        .content(AsyncView::<TextView>::new(s, || AsyncState::Pending))
                        .max_width(32);
                    s.add_layer(loading_view);
                }))
                .unwrap();
        }

        thread::spawn(
            move || match client.get_matched_stories(&query, by_date, page) {
                Err(err) => {
                    warn!(
                        "failed to get stories matching the query '{}': {:#?}",
                        query, err
                    );
                }
                Ok(stories) => {
                    // if the query used to search for "stories"
                    // matches the current query, update view and force redrawing
                    if *self_query.read().unwrap().0 == query {
                        let mut self_stories = self_stories.write().unwrap();

                        *self_stories = stories;
                        self_query.write().unwrap().1 = true;

                        // send an empty callback to force redrawing
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

    pub fn add_char(&mut self, c: char) {
        self.page = 0;
        self.query.write().unwrap().0.push(c);
        self.query.write().unwrap().1 = true;
        self.update_matched_stories();
    }

    pub fn del_char(&mut self) {
        self.page = 0;
        self.query.write().unwrap().0.pop();
        self.query.write().unwrap().1 = true;
        self.update_matched_stories();
    }

    pub fn toggle_by_date(&mut self) {
        self.page = 0;
        self.by_date = !self.by_date;
        self.update_matched_stories();
    }

    pub fn update_page(&mut self, next_page: bool) {
        if next_page {
            self.page += 1;
            self.update_matched_stories();
        } else if self.page > 0 {
            self.page -= 1;
            self.update_matched_stories();
        }
    }

    pub fn new(client: &hn_client::HNClient, cb_sink: CbSink) -> Self {
        let view = Self::get_search_view(SearchViewMode::Search, "", false, 0, vec![], client);
        let stories = Arc::new(RwLock::new(vec![]));
        let query = Arc::new(RwLock::new((String::new(), false)));
        SearchView {
            by_date: false,
            page: 0,
            client: client.clone(),
            mode: SearchViewMode::Search,
            query,
            view,
            stories,
            cb_sink,
        }
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

    fn wrap_draw(&self, printer: &Printer) {
        self.view.draw(printer);
    }
}

/// Return a main view of a SearchView displaying the matched story list with a search bar.
/// The main view of a SearchView is a View without status bar or footer.
fn get_search_main_view(client: &hn_client::HNClient, cb_sink: CbSink) -> impl View {
    OnEventView::new(SearchView::new(&client, cb_sink))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| {
            match s.mode {
                SearchViewMode::Navigation => None,
                SearchViewMode::Search => {
                    match *e {
                        Event::Char(c) => {
                            s.add_char(c);
                            None
                        }
                        Event::Key(Key::Backspace) => {
                            s.del_char();
                            None
                        }
                        // ignore all keys that move the focus out of the search bar
                        Event::Key(Key::Up)
                        | Event::Key(Key::Down)
                        | Event::Key(Key::PageUp)
                        | Event::Key(Key::PageDown)
                        | Event::Key(Key::Tab) => Some(EventResult::Ignored),
                        _ => None,
                    }
                }
            }
        })
        // vim-like switch mode key shortcuts
        .on_pre_event_inner(Event::Key(Key::Esc), |s, _| match s.mode {
            SearchViewMode::Navigation => None,
            SearchViewMode::Search => {
                s.mode = SearchViewMode::Navigation;
                s.view.set_focus_index(1).unwrap_or_else(|_| {});
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('i', |s, _| match s.mode {
            SearchViewMode::Search => None,
            SearchViewMode::Navigation => {
                s.mode = SearchViewMode::Search;
                s.view.set_focus_index(0).unwrap_or_else(|_| {});
                Some(EventResult::Consumed(None))
            }
        })
        .on_pre_event_inner('d', |s, _| {
            if let SearchViewMode::Navigation = s.mode {
                s.toggle_by_date();
            }
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('n', |s, _| {
            if let SearchViewMode::Navigation = s.mode {
                s.update_page(true);
            }
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('p', |s, _| {
            if let SearchViewMode::Navigation = s.mode {
                s.update_page(false);
            }
            Some(EventResult::Consumed(None))
        })
}

/// Return a view representing a SearchView that searches stories with queries
pub fn get_search_view(client: &hn_client::HNClient, cb_sink: CbSink) -> impl View {
    let client = client.clone();
    let main_view = get_search_main_view(&client, cb_sink);
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc("Search View"))
        .child(main_view)
        .child(construct_footer_view::<SearchView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('h') | Event::AltChar('h') => true,
            _ => false,
        }),
        |s| {
            s.add_layer(SearchView::construct_help_view());
        },
    )
}

/// Add SearchView as a new layer to the main Cursive View
pub fn add_search_view_layer(s: &mut Cursive, client: &hn_client::HNClient) {
    let cb_sink = s.cb_sink().clone();
    s.screen_mut()
        .add_transparent_layer(Layer::new(get_search_view(&client, cb_sink)));
}
