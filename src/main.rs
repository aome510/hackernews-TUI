// modules
pub mod hn_client;
pub mod prelude;
pub mod view;

use prelude::*;

fn main() {
    env_logger::init();

    let mut s = cursive::default();

    // load theme
    s.load_toml(include_str!("../theme.toml")).unwrap();

    let client = hn_client::HNClient::new().unwrap();
    story_view::add_story_view_layer(&mut s, &client);

    // universal shortcuts
    s.add_global_callback(Event::CtrlChar('f'), {
        let client = client.clone();
        move |s| {
            story_view::add_story_view_layer(s, &client);
        }
    });
    s.add_global_callback(Event::CtrlChar('s'), {
        let client = client.clone();
        move |s| {
            search_view::add_search_view_layer(s, &client);
        }
    });
    s.add_global_callback(Event::CtrlChar('h'), |s| {
        s.add_layer(DefaultHelpView::construct_help_view())
    });
    s.add_global_callback(Event::CtrlChar('q'), |s| s.quit());

    s.run();
}
