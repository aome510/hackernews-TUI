// modules
mod keybindings;
mod theme;

// re-export
pub use keybindings::*;
pub use theme::*;

use config_parser2::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, ConfigParse)]
/// Config is a struct storing the application's configurations
pub struct Config {
    pub page_scrolling: bool,
    pub use_pacman_loading: bool,
    pub client_timeout: u64,
    pub url_open_command: Command,
    pub article_parse_command: Command,

    pub theme: theme::Theme,
    pub keymap: keybindings::KeyMap,
}

impl Config {
    /// parse config from a file
    pub fn from_config_file(file_path: &str) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(file_path)?;
        let value = toml::from_str::<toml::Value>(&config_str)?;
        let mut config = Self::default();
        config.parse(value)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            page_scrolling: true,
            use_pacman_loading: true,
            url_open_command: Command {
                command: "open".to_string(),
                options: vec![],
            },
            article_parse_command: Command {
                command: "article_md".to_string(),
                options: vec!["--format".to_string(), "html".to_string()],
            },
            client_timeout: 32,
            theme: theme::Theme::default(),
            keymap: keybindings::KeyMap::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub command: String,
    pub options: Vec<String>,
}

config_parser_impl!(Command);

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.command, self.options.join(" ")))
    }
}

static CONFIG: once_cell::sync::OnceCell<Config> = once_cell::sync::OnceCell::new();

/// loads the configuration options from a config file.
/// If failed to find/process the file, loads the default options.
pub fn load_config(config_file_path: Option<&str>) {
    // if no config file is specified, use the default value ($HOME/.config/hn-tui.toml)
    let config_file_path = if let Some(path) = config_file_path {
        Some(path.to_string())
    } else {
        dirs_next::home_dir().map(|path| format!("{}/.config/hn-tui.toml", path.to_str().unwrap()))
    };

    let config = match config_file_path {
        None => Config::default(),
        Some(config_file_path) => match Config::from_config_file(&config_file_path) {
            Err(err) => {
                tracing::error!(
                    "failed to load configurations from the file {}: {} \
                     \n...Use the default configurations instead",
                    config_file_path,
                    err
                );
                Config::default()
            }
            Ok(config) => config,
        },
    };

    tracing::info!("application's configurations: {:?}", config);

    init_config(config);
}

fn init_config(config: Config) {
    CONFIG.set(config).unwrap_or_else(|_| {
        panic!("failed to set up the application's configurations");
    });
}

pub fn get_config() -> &'static Config {
    CONFIG.get().unwrap()
}
