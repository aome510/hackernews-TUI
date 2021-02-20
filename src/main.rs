use anyhow::Result;

// modules
mod hn_client;
mod view;

#[tokio::main]
async fn main() {
    if let Err(err) = start().await {
        eprintln!("{:?}", err);
    }
}

async fn start() -> Result<()> {
    let client = hn_client::HNClient::new();
    let stories = client.get_top_stories().await?;
    let mut siv = cursive::default();

    // load theme
    siv.load_toml(include_str!("../theme.toml")).unwrap();

    siv.add_layer(view::get_story_view(stories, &client)?);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    Ok(())
}
