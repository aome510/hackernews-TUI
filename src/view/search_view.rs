use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use super::event_view::ListEventView;
use crate::prelude::*;
use std::thread;

use super::story_view;

/// SearchView is a view used to search for stories
pub struct SearchView {
    query: String,
    view: LinearLayout,
    client: hn_client::HNClient,
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);
}

impl SearchView {
    fn get_matched_stories_view(query: &str, client: &hn_client::HNClient) -> impl View {
        let client = client.clone();
        ErrorViewWrapper::new(match client.get_matched_stories(query) {
            Err(err) => ErrorViewEnum::Err(error_view::get_error_view(
                format!("failed to get matched stories with query {}", query),
                err,
                &client,
            )),
            Ok(stories) => ErrorViewEnum::Ok({
                OnEventView::new(story_view::StoryView::new(stories))
                    .on_pre_event_inner(Key::Enter, {
                        move |s, _| {
                            let id = s.get_inner().get_focus_index();
                            let story = s.stories[id].clone();
                            Some(EventResult::with_cb({
                                let client = client.clone();
                                move |s| {
                                    let async_view =
                                        async_view::get_comment_view_async(s, &client, &story);
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
            }),
        })
    }

    fn get_query_text_view() -> impl View {
        TextArea::new().full_width().fixed_height(1)
    }

    fn update_matched_stories_view(&mut self) {
        self.view.remove_child(1).unwrap();
        debug!("query: {}", self.query);
        thread::spawn(|| {
            self.view.add_child(SearchView::get_matched_stories_view(
                &self.query,
                &self.client,
            ));
        });
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
        self.update_matched_stories_view();
    }

    pub fn del_char(&mut self) {
        self.query.pop();
        self.update_matched_stories_view();
    }

    pub fn new(client: &hn_client::HNClient) -> Self {
        SearchView {
            client: client.clone(),
            query: "".to_string(),
            view: LinearLayout::vertical()
                .child(SearchView::get_query_text_view())
                .child(SearchView::get_matched_stories_view("", &client)),
        }
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
