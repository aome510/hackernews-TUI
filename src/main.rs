use anyhow::Result;
use view::story_view::get_story_view;

// modules
pub mod hn_client;
pub mod view;

fn main() {
    if let Err(err) = start() {
        log::error!("{:#?}", err)
    }
}

fn start() -> Result<()> {
    env_logger::init();

    let client = hn_client::HNClient::new()?;
    let stories = client.get_top_stories()?;
    let mut siv = cursive::default();

    // load theme
    siv.load_toml(include_str!("../theme.toml")).unwrap();

    siv.add_layer(get_story_view(stories, &client));
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    Ok(())
}
