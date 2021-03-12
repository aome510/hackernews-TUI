use super::error_view::{self, ErrorViewEnum, ErrorViewWrapper};
use super::event_view::ListEventView;
use crate::prelude::*;

use super::story_view;

/// SearchView is a view used to search for stories
pub struct SearchView {
    query: String,
    view: LinearLayout,
}

impl ViewWrapper for SearchView {
    wrap_impl!(self.view: LinearLayout);
}

impl SearchView {
    fn get_matched_stories_view(query: &str, client: &hn_client::HNClient) -> impl View {
        let client = client.clone();
        ErrorViewWrapper::new(match client.get_matched_stories("") {
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

    fn get_query_text_view(query: &str) -> impl View {
        Layer::with_color(
            TextView::new(query.to_string())
                .full_width()
                .fixed_height(1),
            ColorStyle::back(Color::Light(BaseColor::White)),
        )
    }

    pub fn new(client: &hn_client::HNClient) -> Self {
        SearchView {
            query: "".to_string(),
            view: LinearLayout::vertical()
                .child(SearchView::get_query_text_view(""))
                .child(SearchView::get_matched_stories_view("", &client)),
        }
    }
}

pub fn get_search_view(client: &hn_client::HNClient) -> impl View {
    let client = client.clone();
    OnEventView::new(SearchView::new(&client)).on_event(Event::AltChar('f'), move |s| {
        s.pop_layer();
        let async_view = async_view::get_story_view_async(s, &client);
        s.screen_mut().add_transparent_layer(Layer::new(async_view));
    })
}
