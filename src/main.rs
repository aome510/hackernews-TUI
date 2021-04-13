// modules
mod config;
pub mod hn_client;
pub mod prelude;
pub mod view;

use clap::*;
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

fn load_config(config_file_path: Option<&str>) {
    // if no config file is specified, use the default value
    // at $HOME/.config/hn-tui.toml
    let config_file_path = match config_file_path {
        None => {
            let home_dir = dirs::home_dir();
            match home_dir {
                None => None,
                Some(path) => Some(format!("{}/.config/hn-tui.toml", path.to_str().unwrap())),
            }
        }
        Some(path) => Some(path.to_string()),
    };

    let config = if config_file_path.is_none() {
        config::Config::default()
    } else {
        let config_file_path = config_file_path.unwrap();
        match config::Config::from_config_file(&config_file_path) {
            Err(err) => {
                error!(
                    "failed to load the application config from the file: {}: {:#?}",
                    config_file_path, err
                );
                config::Config::default()
            }
            Ok(config) => config,
        }
    };
    config::CONFIG.set(config).unwrap();
}

fn run() {
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

fn main() {
    env_logger::init();

    // parse command line arguments
    let matches = App::new("hackernews-tui")
        .version("0.4.1")
        .author("Thang Pham <phamducthang1234@gmail>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Path to the application's config file (default: ~/.config/hn-tui.toml)"),
        )
        .get_matches();

    load_config(matches.value_of("config"));
    run();
}
