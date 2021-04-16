use std::{
    sync::{Arc, RwLock},
    thread,
};

use super::help_view::*;
use super::text_view;
use super::utils::*;

use crate::prelude::*;

pub enum SearchViewMode {
    Navigation,
    Search,
}

/// SearchView is a view displaying a search bar and
/// a matched story list matching a query string
pub struct SearchView {
    // ("query_text", "need_update_view") pair
    by_date: bool,
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
    ) -> impl View {
        let client = client.clone();
        story_view::get_story_main_view(stories, &client, 0).full_height()
    }

    fn get_query_text_view(query: String) -> impl View {
        let mut style_string = StyledString::styled(
            format!("Search Query:"),
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
        query: &str,
        stories: Vec<hn_client::Story>,
        client: &hn_client::HNClient,
    ) -> LinearLayout {
        LinearLayout::vertical()
            .child(Self::get_query_text_view(query.to_string()))
            .child(Self::get_matched_stories_view(stories, client))
    }

    fn update_view(&mut self) {
        if self.query.read().unwrap().1 {
            self.view = Self::get_search_view(
                &self.query.read().unwrap().0.clone(),
                self.stories.read().unwrap().clone(),
                &self.client,
            );
            // every time view is updated, reset SearchViewMode to insert/search mode
            self.mode = SearchViewMode::Search;
            self.query.write().unwrap().1 = false;
        }
    }

    fn update_matched_stories(&mut self) {
        let self_stories = Arc::clone(&self.stories);
        let self_query = Arc::clone(&self.query);

        let client = self.client.clone();
        let query = self.query.read().unwrap().0.clone();
        let cb_sink = self.cb_sink.clone();
        let by_date = self.by_date;

        thread::spawn(move || match client.get_matched_stories(&query, by_date) {
            Err(err) => {
                error!(
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
                    cb_sink.send(Box::new(|_| {})).unwrap();
                }
            }
        });
    }

    pub fn add_char(&mut self, c: char) {
        self.query.write().unwrap().0.push(c);
        self.query.write().unwrap().1 = true;
        self.update_matched_stories();
    }

    pub fn del_char(&mut self) {
        self.query.write().unwrap().0.pop();
        self.query.write().unwrap().1 = true;
        self.update_matched_stories();
    }

    pub fn toggle_by_date(&mut self) {
        self.by_date = !self.by_date;
        self.update_matched_stories();
    }

    pub fn new(client: &hn_client::HNClient, cb_sink: CbSink) -> Self {
        let view = Self::get_search_view("", vec![], client);
        let stories = Arc::new(RwLock::new(vec![]));
        let query = Arc::new(RwLock::new((String::new(), false)));
        SearchView {
            by_date: false,
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
        .on_pre_event_inner(
            EventTrigger::from_fn(|e| match e {
                Event::CtrlChar('d') | Event::AltChar('d') => true,
                _ => false,
            }),
            |s, _| {
                s.toggle_by_date();
                Some(EventResult::Consumed(None))
            },
        )
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
    s.pop_layer();
    s.screen_mut()
        .add_transparent_layer(Layer::new(get_search_view(&client, cb_sink)));
}
