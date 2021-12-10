use super::{help_view::*, story_view::StoryView, text_view::EditableTextView};
use crate::prelude::*;

#[derive(Copy, Clone)]
pub enum SearchViewMode {
    Navigation,
    Search,
}

pub struct MatchedStories {
    pub query: String,
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
    // TODO: add back loading widget for search box?
    cb_sink: CbSink,
}

impl SearchView {
    pub fn new(client: &'static client::HNClient, cb_sink: CbSink) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        let view = LinearLayout::vertical()
            .child(EditableTextView::new())
            .child(StoryView::new(vec![], 0).full_height());

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

    pub fn get_search_bar_view_mut(&mut self) -> Option<&mut EditableTextView> {
        self.view
            .get_child_mut(0)?
            .downcast_mut::<EditableTextView>()
    }

    pub fn retrieve_matched_stories(&mut self) {
        let query = match self.get_search_bar_view_mut() {
            None => return,
            Some(view) => view.get_text(),
        };
        info!("retrieve matched stories for {}", query);
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
                    sender.send(MatchedStories { query, stories }).unwrap();
                    // send a dummy callback to `cb_sink`
                    cb_sink.send(Box::new(move |_| {})).unwrap();
                }
                Err(err) => {
                    warn!("failed to get matched stories (query={}): {}", query, err);
                }
            },
        );
    }

    pub fn try_update_view(&mut self) {
        let query = match self.get_search_bar_view_mut() {
            None => return,
            Some(view) => view.get_text(),
        };
        info!("current query: {}", query);
        while let Ok(matched_stories) = self.receiver.try_recv() {
            info!("got match stories for {}", matched_stories.query);
            if query == matched_stories.query {
                self.update_stories_view(matched_stories.stories);
            }
        }
    }

    fn update_stories_view(&mut self, stories: Vec<client::Story>) {
        self.view.remove_child(1);
        let starting_id = config::get_config().client.story_limit.search * self.page;
        self.view
            .add_child(StoryView::new(stories, starting_id).full_height());
    }
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);

    fn wrap_layout(&mut self, size: Vec2) {
        info!("`wrap_layout` is called...");
        self.try_update_view();
        self.view.layout(size);
    }
}

/// Return a main view of a SearchView displaying the matched story list with a search bar.
/// The main view of a SearchView is a View without status bar or footer.
fn get_search_main_view(client: &'static client::HNClient, cb_sink: CbSink) -> impl View {
    // let story_view_keymap = config::get_story_view_keymap().clone();
    let search_view_keymap = config::get_search_view_keymap().clone();

    OnEventView::new(SearchView::new(client, cb_sink))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| match s.mode {
            SearchViewMode::Navigation => None,
            SearchViewMode::Search => {
                info!("received an event when in search mode...");
                let view = s.get_search_bar_view_mut()?;
                match *e {
                    Event::Char(c) => {
                        view.add_char(c);
                        s.retrieve_matched_stories();
                    }
                    _ => {
                        // handle editing shortcuts when in the search mode
                        let edit_keymap = config::get_edit_keymap().clone();
                        if *e == edit_keymap.backward_delete_char.into() {
                            view.del_char();
                            s.retrieve_matched_stories();
                        } else if *e == edit_keymap.move_cursor_left.into() {
                            view.move_cursor_left();
                        } else if *e == edit_keymap.move_cursor_right.into() {
                            view.move_cursor_right();
                        } else if *e == edit_keymap.move_cursor_to_begin.into() {
                            view.move_cursor_to_begin();
                        } else if *e == edit_keymap.move_cursor_to_end.into() {
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
                s.mode = SearchViewMode::Navigation;
                s.view.set_focus_index(1).unwrap_or_else(|_| {});
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
    // TODO: implement page/filtering
    // .on_pre_event_inner(story_view_keymap.toggle_sort_by, |s, _| match s.mode {
    //     SearchViewMode::Navigation => s.toggle_by_date(),
    //     SearchViewMode::Search => Some(EventResult::Ignored),
    // })
    // .on_pre_event_inner(story_view_keymap.next_page, |s, _| match s.mode {
    //     SearchViewMode::Navigation => s.update_page(true),
    //     SearchViewMode::Search => Some(EventResult::Ignored),
    // })
    // .on_pre_event_inner(story_view_keymap.prev_page, |s, _| match s.mode {
    //     SearchViewMode::Navigation => s.update_page(false),
    //     SearchViewMode::Search => Some(EventResult::Ignored),
    // })
}

/// Return a view representing a SearchView that searches stories with queries
pub fn get_search_view(client: &'static client::HNClient, cb_sink: CbSink) -> impl View {
    let main_view = get_search_main_view(client, cb_sink);
    let mut view = LinearLayout::vertical()
        .child(utils::get_status_bar_with_desc("Search View"))
        .child(main_view)
        .child(utils::construct_footer_view::<SearchView>());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(config::get_global_keymap().open_help_dialog.clone(), |s| {
        s.add_layer(SearchView::construct_help_view());
    })
}

/// Add a SearchView as a new layer to the main Cursive View
pub fn add_search_view_layer(s: &mut Cursive, client: &'static client::HNClient) {
    let cb_sink = s.cb_sink().clone();
    s.screen_mut()
        .add_transparent_layer(Layer::new(get_search_view(client, cb_sink)));
}
