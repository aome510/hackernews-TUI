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

fn load_config(config_file_path: Option<&str>) {
    // if no config file is specified, use the default value
    // at $HOME/.config/hn-tui.toml
    let config_file_path = if let Some(path) = config_file_path {
        Some(path.to_string())
    } else {
        dirs_next::home_dir().map(|path| format!("{}/.config/hn-tui.toml", path.to_str().unwrap()))
    };

    let config = match config_file_path {
        None => config::Config::default(),
        Some(config_file_path) => match config::Config::from_config_file(&config_file_path) {
            Err(err) => {
                error!(
                    "failed to load the application config from the file {}: {:#?} \
                     \n...Use the default configurations instead",
                    config_file_path, err
                );
                config::Config::default()
            }
            Ok(config) => config,
        },
    };

    init_config(config);
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

    // use buffered_backend to fix the flickering issue
    // when using cursive with crossterm_backend
    // (https://github.com/gyscos/Cursive/issues/142)
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
        .version("0.7.3")
        .author("Thang Pham <phamducthang1234@gmail>")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Path to the application's config file (default: ~/.config/hn-tui.toml)")
                .next_line_help(true),
        )
        .get_matches();

    load_config(matches.value_of("config"));
    run();
}
