// modules
pub mod client;
pub mod config;
pub mod prelude;
pub mod utils;
pub mod view;

use clap::*;
use prelude::*;
use view::help_view::HasHelpView;

macro_rules! set_up_switch_view_shortcut {
    ($key:expr,$tag:expr,$s:expr,$client:expr) => {
        $s.set_on_post_event($key, move |s| {
            view::story_view::add_story_view_layer(
                s,
                $client,
                $tag,
                true,
                0,
                client::StoryNumericFilters::default(),
                false,
            );
        });
    };
}

fn set_up_global_callbacks(s: &mut Cursive, client: &'static client::HNClient) {
    s.clear_global_callbacks(Event::CtrlChar('c'));

    let global_keymap = config::get_global_keymap().clone();

    // .............................................................
    // global shortcuts for switching between different Story Views
    // .............................................................

    set_up_switch_view_shortcut!(global_keymap.goto_front_page_view, "front_page", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_all_stories_view, "story", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_ask_hn_view, "ask_hn", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_show_hn_view, "show_hn", s, client);
    set_up_switch_view_shortcut!(global_keymap.goto_jobs_view, "job", s, client);

    // custom navigation shortcuts
    config::get_config()
        .keymap.custom_keymaps
        .iter()
        .for_each(|data| {
            s.set_on_post_event(data.key.clone(), move |s| {
                view::story_view::add_story_view_layer(
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

    // ............................................
    // end of navigation shortcuts for Story Views
    // ............................................

    s.set_on_post_event(global_keymap.goto_previous_view, |s| {
        if s.screen_mut().len() > 1 {
            s.pop_layer();
        }
    });

    s.set_on_post_event(global_keymap.goto_search_view, move |s| {
        view::search_view::add_search_view_layer(s, client);
    });

    s.set_on_post_event(global_keymap.open_help_dialog, |s| {
        s.add_layer(view::help_view::DefaultHelpView::construct_help_view())
    });

    s.set_on_post_event(global_keymap.quit, |s| s.quit());
}

fn run() {
    let mut s = cursive::default();

    let theme = config::get_config_theme();
    s.update_theme(|t| {
        t.palette.set_color("view", theme.palette.background.into());
        t.palette
            .set_color("primary", theme.palette.foreground.into());
        t.palette
            .set_color("title_primary", theme.palette.foreground.into());
        t.palette
            .set_color("highlight", theme.palette.selection_background.into());
        t.palette
            .set_color("highlight_text", theme.palette.selection_foreground.into());
    });

    // setup HN Client
    let client = client::init_client();
    set_up_global_callbacks(&mut s, client);

    // render `front_page` story view as the application's default view
    view::story_view::add_story_view_layer(
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
fn init_logging(log_folder_path: Option<&str>) {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }

    // if no log folder path is specified, use the default value (`$HOME/.cache`)
    let log_folder_path = match log_folder_path {
        Some(path) => path.into(),
        None => dirs_next::home_dir()
            .expect("failed to get user's home directory")
            .join(".cache"),
    };

    if !log_folder_path.exists() {
        std::fs::create_dir_all(&log_folder_path).expect("failed to create a log folder");
    }

    let log_file =
        std::fs::File::create(log_folder_path.join("hn-tui.log")).unwrap_or_else(|err| {
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
        .version("0.8.1")
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
                .value_name("FOLDER")
                .help("Path to a folder to store application's logs (default: $HOME/.cache)")
                .next_line_help(true),
        )
        .get_matches();

    init_logging(args.value_of("log"));
    config::load_config(args.value_of("config"));
    run();
}
