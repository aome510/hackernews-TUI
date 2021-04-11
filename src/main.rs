// modules
pub mod hn_client;
pub mod prelude;
pub mod view;

use prelude::*;

fn set_up_global_callbacks(s: &mut Cursive, client: &hn_client::HNClient) {
    s.set_on_post_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('f') | Event::AltChar('f') => true,
            _ => false,
        }),
        {
            let client = client.clone();
            move |s| {
                story_view::add_story_view_layer(s, &client);
            }
        },
    );

    s.set_on_post_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('s') | Event::AltChar('s') => true,
            _ => false,
        }),
        {
            let client = client.clone();
            move |s| {
                search_view::add_search_view_layer(s, &client);
            }
        },
    );

    s.set_on_post_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('h') | Event::AltChar('h') => true,
            _ => false,
        }),
        |s| s.add_layer(DefaultHelpView::construct_help_view()),
    );

    s.set_on_post_event(
        EventTrigger::from_fn(|e| match e {
            Event::CtrlChar('q') | Event::AltChar('q') => true,
            _ => false,
        }),
        |s| s.quit(),
    );
}

fn main() {
    env_logger::init();

    let mut s = cursive::default();

    // load theme
    s.load_toml(include_str!("../theme.toml")).unwrap();

    let client = hn_client::HNClient::new().unwrap();
    story_view::add_story_view_layer(&mut s, &client);

    set_up_global_callbacks(&mut s, &client);

    // use buffered_backend
    let crossterm_backend = backends::crossterm::Backend::init().unwrap();
    let buffered_backend = Box::new(cursive_buffered_backend::BufferedBackend::new(
        crossterm_backend,
    ));
    let mut app = CursiveRunner::new(s, buffered_backend);

    app.run();
}
