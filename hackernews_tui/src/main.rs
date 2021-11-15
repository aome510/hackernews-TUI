// modules
pub mod client;
pub mod config;
pub mod prelude;
pub mod utils;
pub mod view;

use clap::*;
use prelude::*;

macro_rules! set_up_switch_view_shortcut {
    ($key:expr,$tag:expr,$s:expr,$client:expr) => {
        $s.set_on_post_event($key, move |s| {
            story_view::add_story_view_layer(
                s,
                $client,
                $tag,
                false,
                0,
                client::StoryNumericFilters::default(),
                false,
            );
        });
    };
}

fn set_up_global_callbacks(s: &mut Cursive, client: &'static client::HNClient) {
    s.clear_global_callbacks(Event::CtrlChar('c'));

    let global_keymap = get_global_keymap().clone();

    // .............................................................
    // global shortcuts for switching between different Story Views
    // .............................................................

    set_up_switch_view_shortcut!(global_keymap.goto_front_page_view, "front_page", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_all_stories_view, "story", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_ask_hn_view, "ask_hn", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_show_hn_view, "show_hn", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_jobs_view, "job", s, client);

    // custom navigation shortcuts
    let custom_keymap = get_custom_keymap();
    custom_keymap
        .custom_view_navigation
        .iter()
        .for_each(|data| {
            s.set_on_post_event(data.key.clone(), move |s| {
                story_view::add_story_view_layer(
                    s,
                    client,
                    &data.tag,
                    data.by_date,
                    0,
                    data.numeric_filters,
                    false,
                );
            });
        });

    // .........................................
    // end of navigation shortcuts for StoryView
    // .........................................

    s.set_on_post_event(global_keymap.goto_previous_view, |s| {
        if s.screen_mut().len() > 1 {
            s.pop_layer();
        }
    });

    s.set_on_post_event(global_keymap.goto_search_view, move |s| {
        search_view::add_search_view_layer(s, client);
    });

    s.set_on_post_event(global_keymap.open_help_dialog, |s| {
        s.add_layer(DefaultHelpView::construct_help_view())
    });

    s.set_on_post_event(global_keymap.quit, |s| s.quit());
}

fn run() {
    let mut s = cursive::default();

    // update cursive's default theme
    let config_theme = get_config().theme.clone();
    s.update_theme(|theme| {
        config_theme.update_theme(theme);
    });

    // setup HN Client
    let client = client::init_client();
    set_up_global_callbacks(&mut s, client);

    story_view::add_story_view_layer(
        &mut s,
        client,
        "front_page",
        false,
        0,
        client::StoryNumericFilters::default(),
        false,
    );

    // use `cursive_buffered_backend` to fix the flickering issue
    // when using `cursive` with `crossterm_backend` (https://github.com/gyscos/Cursive/issues/142)
    let crossterm_backend = backends::crossterm::Backend::init().unwrap();
    let buffered_backend = Box::new(cursive_buffered_backend::BufferedBackend::new(
        crossterm_backend,
    ));
    let mut app = CursiveRunner::new(s, buffered_backend);

    app.run();
}

/// initialize application logging
fn init_logging(log_file_path: Option<&str>) {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    // if no log file path is specified, use the default value (`$HOME/.cache/hn-tui.log`)
    let log_file_path = match log_file_path {
        Some(path) => path.into(),
        None => dirs_next::home_dir()
            .expect("failed to get user's cache directory")
            .join(".cache")
            .join("hn-tui.log"),
    };
    let log_file = std::fs::File::create(log_file_path).unwrap_or_else(|err| {
        panic!("failed to create application's log file: {}", err);
    });
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(log_file))
        .init();
}

fn main() {
    // parse command line arguments
    let args = App::new("hackernews-tui")
        .version("0.7.3")
        .author("Thang Pham <phamducthang1234@gmail>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Path to the application's config file (default: $HOME/.config/hn-tui.toml)")
                .next_line_help(true),
        )
        .arg(
            Arg::with_name("log")
                .short("l")
                .long("log")
                .value_name("FILE")
                .help("Path to the application's log file (default: $HOME/.cache/hn-tui.log)")
                .next_line_help(true),
        )
        .get_matches();

    init_logging(args.value_of("log"));
    load_config(args.value_of("config"));
    run();
}
