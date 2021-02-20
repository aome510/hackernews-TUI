use cursive::{
    traits::*,
    event::EventResult,
    views::{SelectView, OnEventView},
};

// modules
mod hn_client;

use hn_client::HNClient;

#[tokio::main]
async fn main() {
    let client = HNClient::new();
    match client.get_top_stories().await {
        Ok(stories) => {
            let mut siv = cursive::default();

            // load theme
            siv.load_toml(include_str!("../theme.toml")).unwrap();

            let stories_view = SelectView::new()
                .with_all(stories
                          .into_iter()
                          .enumerate()
                          .map(|(i, story)| (format!("title: {}, url: {}", story.title, story.url), i)));

            // add "j" and "k" for moving down and up the story list
            let stories_view = OnEventView::new(stories_view)
                .on_pre_event_inner('k', |s, _| {
                    let cb = s.select_up(1);
                    Some(EventResult::Consumed(Some(cb)))
                })
                .on_pre_event_inner('j', |s, _| {
                    let cb = s.select_down(1);
                    Some(EventResult::Consumed(Some(cb)))
                });


            siv.add_layer(
                stories_view.scrollable()
            );
            siv.add_global_callback('q', |s| s.quit());
            siv.run();
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
}
