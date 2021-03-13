use super::event_view::ListEventView;
use super::story_view;
use crate::prelude::*;
use std::{
    sync::{Arc, RwLock},
    thread,
};

/// SearchView is a view used to search for stories
pub struct SearchView {
    query: String,
    stories: Arc<RwLock<Vec<hn_client::Story>>>,
    view: LinearLayout,
    client: hn_client::HNClient,
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
        TextArea::new().content(query).full_width().fixed_height(1)
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
        self.view = Self::get_search_view(
            &self.query,
            self.stories.read().unwrap().clone(),
            &self.client,
        );
    }

    fn update_matched_stories(&mut self) {
        debug!("query: {}", self.query);

        let self_stories = Arc::clone(&self.stories);
        let client = self.client.clone();
        let query = self.query.to_string();
        thread::spawn(move || match client.get_matched_stories(&query) {
            Err(err) => {
                debug!(
                    "failed to get stories matching the query '{}': {:#?}",
                    query, err
                );
            }
            Ok(stories) => {
                let mut self_stories = self_stories.write().unwrap();
                *self_stories = stories;
            }
        });
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
        self.update_matched_stories();
    }

    pub fn del_char(&mut self) {
        self.query.pop();
        self.update_matched_stories();
    }

    pub fn new(client: &hn_client::HNClient) -> Self {
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
        SearchView {
            client: client.clone(),
            query: "".to_string(),
            view,
            stories,
        }
    }
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);

    fn wrap_needs_relayout(&self) -> bool {
        true
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.update_view();
        self.view.layout(size)
    }
}

pub fn get_search_view(client: &hn_client::HNClient) -> impl View {
    let client = client.clone();
    OnEventView::new(SearchView::new(&client))
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
