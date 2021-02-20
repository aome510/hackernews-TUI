use super::hn_client::*;
use anyhow::Result;
use cursive::{
    event::EventResult,
    views::{OnEventView, SelectView},
};

fn construct_event_view(view: SelectView<Story>) -> OnEventView<SelectView<Story>> {
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
pub fn get_story_view(stories: Vec<Story>, hn_client: &HNClient) -> Result<OnEventView<SelectView<Story>>> {
    let hn_client = hn_client.clone();
    Ok(construct_event_view(
        SelectView::new()
            .with_all(
                stories
                    .into_iter()
                    .map(|story| (format!("title: {}, url: {}", story.title, story.url), story)),
            )
            .on_submit(move |s, story| {
                s.pop_layer();
                let comments = story.get_all_comments(&hn_client);
                s.add_layer(cursive::views::TextView::new(format!("{:#?}", comments)));
            }),
    ))
}

// pub fn get_comment_view(comments: Vec<Comment>, hn_client: &HNClient) -> OnEventView<SelectView<Comment>> {
// }
