use anyhow::Result;

// modules
mod hn_client;
mod view;

fn main() -> Result<()> {
    let client = hn_client::HNClient::new();
    let stories = client.get_top_stories()?;
    let mut siv = cursive::default();

    // load theme
    siv.load_toml(include_str!("../theme.toml")).unwrap();

    siv.add_layer(view::get_story_view(stories, &client)?);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    Ok(())
}
