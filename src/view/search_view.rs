use super::event_view::ListEventView;
use super::story_view;
use crate::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
};

/// SearchView is a view used to search for stories
pub struct SearchView {
    query: Arc<RwLock<(String, bool)>>,
    stories: Arc<RwLock<Vec<hn_client::Story>>>,

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
        OnEventView::new(story_view::StoryView::new(stories.clone()))
            .on_pre_event_inner(Key::Enter, {
                move |s, _| {
                    let id = s.get_inner().get_focus_index();
                    let story = s.stories[id].clone();
                    Some(EventResult::with_cb({
                        let client = client.clone();
                        move |s| {
                            let async_view = async_view::get_comment_view_async(s, &client, &story);
                            s.pop_layer();
                            s.screen_mut().add_transparent_layer(Layer::new(async_view))
                        }
                    }))
                }
            })
            .on_pre_event_inner(Event::CtrlChar('k'), |s, _| s.focus_up())
            .on_pre_event_inner(Event::CtrlChar('j'), |s, _| s.focus_down())
            .full_height()
            .scrollable()
    }

    fn get_query_text_view(query: String) -> impl View {
        let len = query.len();
        let mut text_area = TextArea::new().content(query);
        text_area.set_cursor(len);
        text_area.full_width().fixed_height(1)
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
        self.query.write().unwrap().1 = false;
        self.update_matched_stories();
    }

    pub fn del_char(&mut self) {
        self.query.write().unwrap().0.pop();
        self.query.write().unwrap().1 = false;
        self.update_matched_stories();
    }

    pub fn new(client: &hn_client::HNClient, cb_sink: CbSink) -> Self {
        let stories = match client.get_matched_stories("") {
            Err(err) => {
                warn!(
                    "failed to get stories matching the query '{}': {:#?}",
                    "", err
                );
                vec![]
            }
            Ok(stories) => stories,
        };
        let view = Self::get_search_view("", stories.clone(), client);
        let stories = Arc::new(RwLock::new(stories));
        let query = Arc::new(RwLock::new(("".to_string(), false)));
        SearchView {
            client: client.clone(),
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
}

/// Return a view represeting a SearchView with registered key-pressed event handlers
pub fn get_search_view(client: &hn_client::HNClient, cb_sink: CbSink) -> impl View {
    let client = client.clone();
    OnEventView::new(SearchView::new(&client, cb_sink))
        .on_event(Event::AltChar('f'), move |s| {
            s.pop_layer();
            let async_view = async_view::get_story_view_async(s, &client);
            s.screen_mut().add_transparent_layer(Layer::new(async_view));
        })
        .on_pre_event_inner(EventTrigger::from_fn(|_| true), |s, e| match *e {
            Event::Char(c) => {
                s.add_char(c);
                None
            }
            Event::Key(Key::Backspace) => {
                s.del_char();
                None
            }
            _ => None,
        })
}
