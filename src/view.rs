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
                let hn_client = hn_client.clone();
                let story = story.clone();
                s.pop_layer();
                let cb_sink = s.cb_sink().clone();
                tokio::spawn(async move {
                    let comments = story.get_all_comments(&hn_client).await.unwrap();
                    cb_sink.send(Box::new(move |s| {
                        s.add_layer(cursive::views::TextView::new(format!("{:#?}", comments)));
                    })).unwrap();
                });
            }),
    ))
}

// pub fn get_comment_view(comments: Vec<Comment>, hn_client: &HNClient) -> OnEventView<SelectView<Comment>> {
// }
