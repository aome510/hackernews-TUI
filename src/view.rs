use super::hn_client::*;
use cursive::{
    event::EventResult,
    traits::Scrollable,
    view::IntoBoxedView,
    views::{OnEventView, SelectView},
};

fn construct_event_view<T: 'static>(view: SelectView<T>) -> OnEventView<SelectView<T>> {
    // add "j" and "k" for moving down and up the story list
    OnEventView::new(view)
        .on_pre_event_inner('k', |s, _| {
            let cb = s.select_up(1);
            Some(EventResult::Consumed(Some(cb)))
        })
        .on_pre_event_inner('j', |s, _| {
            let cb = s.select_down(1);
            Some(EventResult::Consumed(Some(cb)))
        })
}

/// Return a cursive's View from a story list
pub fn get_story_view(stories: Vec<Story>, hn_client: &HNClient) -> impl IntoBoxedView {
    let hn_client = hn_client.clone();
    construct_event_view(
        SelectView::new()
            .with_all(
                stories
                    .into_iter()
                    .map(|story| (format!("{}: {}", story.by, story.title), story)),
            )
            .on_submit(move |s, story| {
                s.pop_layer();
                let comments = story.get_all_comments(&hn_client);
                s.add_layer(get_comment_view(comments));
            }),
    )
    .scrollable()
}

pub fn get_comment_view(comments: Vec<Comment>) -> impl IntoBoxedView {
    construct_event_view(
        SelectView::new().with_all(
            comments
                .into_iter()
                .map(|comment| (format!("{}: {}", comment.by, comment.text), comment)),
        ),
    )
    .scrollable()
}
