use super::story_view;
use super::text_view;
use super::theme::*;
use super::utils::*;
use crate::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
};

/// SearchView is a view used to search for stories
pub struct SearchView {
    // ("query_text", "need_update_view") pair
    query: Arc<RwLock<(String, bool)>>,
    stories: Arc<RwLock<Vec<hn_client::Story>>>,

    // 0: for insert/search mode, 1: for command mode
    mode: bool,

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
        story_view::get_story_main_view(stories, &client)
    }

    fn get_query_text_view(query: String) -> impl View {
        let mut style_string = StyledString::styled(
            format!("Query String:"),
            ColorStyle::new(BOLD_TEXT_COLOR, HIGHLIGHT_COLOR),
        );
        style_string.append_plain(format!(" {}", query));
        Layer::with_color(
            text_view::TextView::new(style_string)
                .fixed_height(1)
                .full_width(),
            ColorStyle::back(Color::Light(BaseColor::White)),
        )
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
            // everytime, we update view, reset to insert/search mode
            self.mode = false;
            self.query.write().unwrap().1 = false;
        }
    }

    fn update_matched_stories(&mut self) {
        let self_stories = Arc::clone(&self.stories);
        let self_query = Arc::clone(&self.query);

        let client = self.client.clone();
        let query = self.query.read().unwrap().0.clone();
        let cb_sink = self.cb_sink.clone();

        thread::spawn(move || match client.get_matched_stories(&query) {
            Err(err) => {
                warn!(
                    "failed to get stories matching the query '{}': {:#?}",
                    query, err
                );
            }
            Ok(stories) => {
                if *self_query.read().unwrap().0 == query {
                    let mut self_stories = self_stories.write().unwrap();

                    *self_stories = stories;
                    self_query.write().unwrap().1 = true;
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

    pub fn new(client: &hn_client::HNClient, cb_sink: CbSink) -> Self {
        let view = Self::get_search_view("", vec![], client);
        let stories = Arc::new(RwLock::new(vec![]));
        let query = Arc::new(RwLock::new(("".to_string(), false)));
        SearchView {
            client: client.clone(),
            mode: false,
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

fn get_main_search_view(client: &hn_client::HNClient, cb_sink: CbSink) -> impl View {
    OnEventView::new(SearchView::new(&client, cb_sink))
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| {
            if s.mode {
                None
            } else {
                match *e {
                    Event::Char(c) => {
                        s.add_char(c);
                        None
                    }
                    Event::Key(Key::Backspace) => {
                        s.del_char();
                        None
                    }
                    // ignore all keys that move the focus out of the text_view
                    Event::Key(Key::Up)
                    | Event::Key(Key::Down)
                    | Event::Key(Key::PageUp)
                    | Event::Key(Key::PageDown)
                    | Event::Key(Key::Tab) => Some(EventResult::Ignored),
                    _ => None,
                }
            }
        })
        .on_pre_event_inner(Event::Key(Key::Esc), |s, _| {
            if !s.mode {
                s.mode = true;
                s.view.set_focus_index(1).unwrap_or_else(|_| {});
                Some(EventResult::Consumed(None))
            } else {
                None
            }
        })
        .on_pre_event_inner('i', |s, _| {
            if s.mode {
                s.mode = false;
                s.view.set_focus_index(0).unwrap_or_else(|_| {});
                Some(EventResult::Consumed(None))
            } else {
                None
            }
        })
}

/// Return a view represeting a SearchView with registered key-pressed event handlers
pub fn get_search_view(client: &hn_client::HNClient, cb_sink: CbSink) -> impl View {
    let client = client.clone();
    let main_view = get_main_search_view(&client, cb_sink);
    let mut view = LinearLayout::vertical()
        .child(get_status_bar_with_desc("Story Search View"))
        .child(main_view)
        .child(construct_footer_view());
    view.set_focus_index(1).unwrap_or_else(|_| {});

    OnEventView::new(view).on_event(Event::AltChar('f'), move |s| {
        s.pop_layer();
        let async_view = async_view::get_story_view_async(s, &client);
        s.screen_mut().add_transparent_layer(Layer::new(async_view));
    })
}
