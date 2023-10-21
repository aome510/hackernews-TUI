// modules
pub mod client;
pub mod config;
pub mod model;
pub mod parser;
pub mod prelude;
pub mod utils;
pub mod view;

const DEFAULT_CONFIG_FILE: &str = "hn-tui.toml";
const DEFAULT_AUTH_FILE: &str = "hn-auth.toml";
const DEFAULT_LOG_FILE: &str = "hn-tui.log";

use clap::*;
use prelude::*;

fn run(auth: Option<config::Auth>, start_id: Option<u32>) {
    // setup HN Client
    let client = client::init_client();

    // login if authentication is specified
    if let Some(auth) = auth {
        if let Err(err) = client.login(&auth.username, &auth.password) {
            tracing::warn!("Failed to login, user={}: {err}", auth.username);
        }
    }

    // setup the application's UI
    let s = view::init_ui(client, start_id);

    // use `cursive_buffered_backend` crate to fix the flickering issue
    // when using `cursive` with `crossterm_backend` (See https://github.com/gyscos/Cursive/issues/142)
    let crossterm_backend = backends::crossterm::Backend::init().unwrap();
    let buffered_backend = Box::new(cursive_buffered_backend::BufferedBackend::new(
        crossterm_backend,
    ));
    let mut app = CursiveRunner::new(s, buffered_backend);

    app.run();
}

/// initialize application logging
fn init_logging(log_dir_str: &str) {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "hackernews_tui=info")
    }

    let log_dir = std::path::PathBuf::from(log_dir_str);
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)
            .unwrap_or_else(|_| panic!("{}", "failed to create a log folder: {log_dir_str}"));
    }

    let log_file = std::fs::File::create(log_dir.join(DEFAULT_LOG_FILE)).unwrap_or_else(|err| {
        panic!("failed to create application's log file: {err}");
    });

    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(log_file))
        .init();
}

/// parse command line arguments
fn parse_args(config_dir: std::path::PathBuf, cache_dir: std::path::PathBuf) -> ArgMatches {
    Command::new("hackernews-tui")
        .version("0.13.4")
        .author("Thang Pham <phamducthang1234@gmail>")
        .about("A Terminal UI to browse Hacker News")
        .arg(
            Arg::new("auth")
                .short('a')
                .long("auth")
                .value_name("FILE")
                .default_value(config_dir.join(DEFAULT_AUTH_FILE).into_os_string())
                .help("Path to the application's authentication file")
                .next_line_help(true),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .default_value(config_dir.join(DEFAULT_CONFIG_FILE).into_os_string())
                .help("Path to the application's config file")
                .next_line_help(true),
        )
        .arg(
            Arg::new("log")
                .short('l')
                .long("log")
                .value_name("FOLDER")
                .default_value(cache_dir.into_os_string())
                .help("Path to a folder to store application's logs")
                .next_line_help(true),
        )
        .arg(
            Arg::new("start_id")
                .short('i')
                .value_parser(clap::value_parser!(u32))
                .help("The Hacker News item's id to start the application with")
                .next_line_help(true),
        )
        .get_matches()
}

fn init_app_dirs() -> (std::path::PathBuf, std::path::PathBuf) {
    let mut config_dir = dirs_next::config_dir().expect("failed to get user's config dir");
    let cache_dir = dirs_next::cache_dir().expect("failed to get user's cache dir");
    let home_dir = dirs_next::home_dir().expect("failed to get user's home dir");

    // Try to find application's config file in the user's config dir.
    // If not found, fallback to use `$HOME/.config` (for backward compability reason)
    if !config_dir.join(DEFAULT_CONFIG_FILE).exists() {
        config_dir = home_dir.join(".config");
    }

    (config_dir, cache_dir)
}

fn init_auth(auth_file_str: &str) -> Option<config::Auth> {
    match config::Auth::from_file(auth_file_str) {
        Ok(auth) => Some(auth),
        Err(err) => {
            tracing::warn!("Failed to get authentication from {}: {err}", auth_file_str);
            None
        }
    }
}

fn main() {
    let (config_dir, cache_dir) = init_app_dirs();
    let args = parse_args(config_dir, cache_dir);

    init_logging(
        args.get_one::<String>("log")
            .expect("`log` argument should have a default value"),
    );

    config::load_config(
        args.get_one::<String>("config")
            .expect("`config` argument should have a default value"),
    );

    let auth = init_auth(
        args.get_one::<String>("auth")
            .expect("`auth` argument should have a default value"),
    );
    let start_id = args.get_one::<u32>("start_id").cloned();
    run(auth, start_id);
}
