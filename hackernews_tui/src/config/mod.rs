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
    pub use_page_scrolling: bool,
    pub use_pacman_loading: bool,
    pub client_timeout: u64,
    pub url_open_command: Command,
    pub article_parse_command: Command,

    pub theme: theme::Theme,
    pub keymap: keybindings::KeyMap,
}

impl Config {
    /// parse config from a file
    pub fn from_config_file<P>(file: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let config_str = std::fs::read_to_string(file)?;
        let value = toml::from_str::<toml::Value>(&config_str)?;
        let mut config = Self::default();
        config.parse(value)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            use_page_scrolling: true,
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

/// loads the configurations from a config file.
/// If failed to find/process the file, uses the default configurations.
pub fn load_config(config_file_str: &str) {
    let config_file = std::path::PathBuf::from(config_file_str);

    let config = match Config::from_config_file(config_file) {
        Err(err) => {
            tracing::error!(
                "failed to load configurations from the file {config_file_str}: {err:#}\
                 \nUse the default configurations instead",
            );
            Config::default()
        }
        Ok(config) => config,
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
