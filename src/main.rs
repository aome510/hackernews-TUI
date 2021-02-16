use cursive::{
    traits::{With, Scrollable},
    views::{TextView, ListView}
};

mod hn_client;

use hn_client::HNClient;

#[tokio::main]
async fn main() {
    let client = HNClient::new();
    if let Ok(stories) = client.get_top_stories().await {
        let stories = stories.into_iter()
            .map(|story| format!("title: {}, url: {}", story.title, story.url))
            .collect::<Vec<String>>();

        let mut siv = cursive::default();

        // load theme
        siv.load_toml(include_str!("../theme.toml")).unwrap();

        siv.add_layer(ListView::new()
                      .with(|list| {
                          stories.iter().enumerate().for_each(|(id, story)| {
                              list.add_child(
                                  &format!("{}.", id),
                                  TextView::new(story)
                              );
                          });
                      })
                      .scrollable());
        siv.add_global_callback('q', |s| s.quit());
        siv.run();
    }
}
