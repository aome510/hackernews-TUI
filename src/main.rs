use cursive::views::TextView;

mod hn_client;

use hn_client::HNClient;

#[tokio::main]
async fn main() {
    let client = HNClient::new();
    if let Ok(stories) = client.get_top_stories().await {
        let stories_str = stories.into_iter()
            .map(|story| format!("title: {}, url: {}", story.title, story.url))
            .collect::<Vec<String>>()
            .join("\n");

        let mut siv = cursive::default();

        // load theme
        siv.load_toml(include_str!("../theme.toml")).unwrap();

        siv.add_layer(TextView::new(stories_str));
        siv.add_global_callback('q', |s| s.quit());
        siv.run();
    }
}
