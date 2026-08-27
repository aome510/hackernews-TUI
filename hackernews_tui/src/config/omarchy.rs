//! Support for following the system's color scheme.
//!
//! [Omarchy](https://omarchy.org) exposes the active theme's palette as a `colors.toml`
//! file inside `$XDG_STATE_HOME/omarchy/current/theme`, which is a symlink updated on
//! every `omarchy theme set`. This module reads that palette, resolves it - mirroring the
//! alias/fallback cascade of the `omarchy-theme-color` helper so that the application ends
//! up with the same colors as any other themed application - and converts it into an
//! application's [`Theme`].

use std::collections::HashMap;

use super::theme::{Color, ComponentStyle, Effect, Palette, Style, Theme};

/// Path of the current theme's color palette, relative to the user's state directory
const COLORS_FILE: &str = "omarchy/current/theme/colors.toml";

/// Constructs the application's theme from the system's color scheme.
///
/// Returns `None` if the system doesn't expose a color scheme or if the exposed
/// color scheme cannot be parsed.
pub fn get_system_theme() -> Option<Theme> {
    let file = colors_file()?;

    match Colors::from_file(&file) {
        Ok(colors) => {
            tracing::info!(
                "using the system color scheme defined in {}",
                file.display()
            );
            Some(colors.into_theme())
        }
        Err(err) => {
            tracing::warn!(
                "failed to parse the system color scheme in {}: {err:#}",
                file.display()
            );
            None
        }
    }
}

/// Returns the path of the current theme's color palette, if such a file exists
fn colors_file() -> Option<std::path::PathBuf> {
    let state_dir = match std::env::var_os("XDG_STATE_HOME") {
        Some(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
        _ => dirs_next::home_dir()?.join(".local").join("state"),
    };

    let file = state_dir.join(COLORS_FILE);
    if file.exists() {
        Some(file)
    } else {
        None
    }
}

/// A system color scheme, as a map of color names to their (hex) values
struct Colors(HashMap<String, String>);

impl Colors {
    fn from_file(file: &std::path::Path) -> anyhow::Result<Self> {
        Self::from_str(&std::fs::read_to_string(file)?)
    }

    fn from_str(colors_str: &str) -> anyhow::Result<Self> {
        let value = toml::from_str::<toml::Value>(colors_str)?;
        let table = match value {
            toml::Value::Table(table) => table,
            _ => anyhow::bail!("expect a TOML table"),
        };

        // a color scheme consists of string values only, any other value is ignored
        let mut colors = Self(
            table
                .into_iter()
                .filter_map(|(key, value)| match value {
                    toml::Value::String(value) => Some((key, value)),
                    _ => None,
                })
                .collect(),
        );

        if colors.get("background").is_none() && colors.get("color0").is_none() {
            anyhow::bail!("no background color defined");
        }
        if colors.get("foreground").is_none() && colors.get("color7").is_none() {
            anyhow::bail!("no foreground color defined");
        }

        colors.resolve();
        Ok(colors)
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.0
            .get(key)
            .map(String::as_str)
            .filter(|value| !value.is_empty())
    }

    fn color(&self, key: &str) -> Option<Color> {
        self.get(key).and_then(Color::try_parse)
    }

    fn color_or(&self, key: &str, default: Color) -> Color {
        self.color(key).unwrap_or(default)
    }

    /// Sets `key` to the value of `from` if the color scheme doesn't define `key`
    fn alias(&mut self, key: &str, from: &str) {
        if self.get(key).is_none() {
            if let Some(value) = self.get(from).map(str::to_owned) {
                self.0.insert(key.to_owned(), value);
            }
        }
    }

    /// Sets `key` to a mix of `from` and `to` if the color scheme doesn't define `key`
    fn derive(&mut self, key: &str, from: &str, to: Rgb, amount: f64) {
        if self.get(key).is_some() {
            return;
        }
        if let Some(from) = self.get(from).and_then(parse_rgb) {
            self.0.insert(key.to_owned(), mix(from, to, amount));
        }
    }

    /// Fills in the colors a theme is allowed to omit.
    ///
    /// The cascade is the one implemented by `omarchy-theme-color`, which every Omarchy
    /// consumer (terminal configs, OSC sequences, previews, ...) resolves through.
    fn resolve(&mut self) {
        // a theme may use the legacy short names for the semantic palette
        for (key, from) in [
            ("background", "bg"),
            ("dark_background", "dark_bg"),
            ("darker_background", "darker_bg"),
            ("lighter_background", "lighter_bg"),
            ("foreground", "fg"),
            ("dark_foreground", "dark_fg"),
            ("light_foreground", "light_fg"),
            ("bright_foreground", "bright_fg"),
        ] {
            self.alias(key, from);
        }

        // a theme predating the semantic palette may only define ANSI names
        for (key, from) in [
            ("background", "color0"),
            ("foreground", "color7"),
            ("red", "color1"),
            ("green", "color2"),
            ("yellow", "color3"),
            ("blue", "color4"),
            ("magenta", "color5"),
            ("cyan", "color6"),
            ("bright_red", "color9"),
            ("bright_green", "color10"),
            ("bright_yellow", "color11"),
            ("bright_blue", "color12"),
            ("bright_magenta", "color13"),
            ("bright_cyan", "color14"),
        ] {
            self.alias(key, from);
        }
        self.alias("magenta", "purple");
        self.alias("bright_magenta", "bright_purple");

        // shades of the background/foreground colors
        self.alias("light_foreground", "foreground");
        self.alias("bright_foreground", "color15");
        self.alias("bright_foreground", "foreground");
        self.alias("lighter_background", "background");
        self.alias("dark_foreground", "color8");
        self.alias("dark_foreground", "foreground");
        self.alias("muted", "color8");
        self.alias("muted", "dark_foreground");
        self.derive("dark_background", "background", BLACK, 0.25);
        self.derive("darker_background", "background", BLACK, 0.5);

        // the selection colors
        self.alias("selection", "selection_background");
        self.alias("selection", "color8");
        self.alias("selection", "background");
        self.alias("selection_background", "selection");
        self.alias("selection_foreground", "bright_foreground");

        // the accent colors
        self.alias("orange", "yellow");
        self.alias("accent", "blue");
        for color in ["red", "yellow", "green", "cyan", "blue", "magenta"] {
            self.derive(&format!("bright_{color}"), color, WHITE, 0.2);
        }
    }

    /// The application's color palette.
    ///
    /// The ANSI colors follow the same mapping Omarchy uses to theme terminal emulators.
    fn palette(&self) -> Palette {
        let default = Palette::default();

        Palette {
            background: self.color_or("background", default.background),
            foreground: self.color_or("foreground", default.foreground),
            selection_background: self
                .color_or("selection_background", default.selection_background),
            selection_foreground: self
                .color_or("selection_foreground", default.selection_foreground),

            black: self.color_or("background", default.black),
            blue: self.color_or("blue", default.blue),
            cyan: self.color_or("cyan", default.cyan),
            green: self.color_or("green", default.green),
            magenta: self.color_or("magenta", default.magenta),
            red: self.color_or("red", default.red),
            white: self.color_or("foreground", default.white),
            yellow: self.color_or("yellow", default.yellow),

            light_black: self.color_or("muted", default.light_black),
            light_white: self.color_or("bright_foreground", default.light_white),
            light_red: self.color_or("bright_red", default.light_red),
            light_magenta: self.color_or("bright_magenta", default.light_magenta),
            light_green: self.color_or("bright_green", default.light_green),
            light_cyan: self.color_or("bright_cyan", default.light_cyan),
            light_blue: self.color_or("bright_blue", default.light_blue),
            light_yellow: self.color_or("bright_yellow", default.light_yellow),
        }
    }

    fn into_theme(self) -> Theme {
        let palette = self.palette();

        // colors of the semantic palette that have no ANSI counterpart, used to keep the
        // component styles from falling back to the hard-coded default colors
        let orange = self.color_or("orange", palette.yellow);
        let muted = self.color_or("muted", palette.light_black);
        let dim = self.color_or("dark_foreground", muted);
        let bright = palette.light_white;

        let component_style = ComponentStyle {
            title_bar: Style::default()
                .back(orange)
                .front(palette.background)
                .effect(Effect::Bold),
            // rendered on top of the title bar, so it needs colors of its own to stand out
            current_story_tag: Style::default()
                .front(palette.selection_foreground)
                .back(palette.selection_background),
            link: Style::default().front(palette.blue),
            link_id: Style::default()
                .front(palette.background)
                .back(palette.yellow),
            matched_highlight: Style::default()
                .front(palette.background)
                .back(palette.yellow),
            single_code_block: Style::default()
                .front(bright)
                .back(palette.selection_background),
            multiline_code_block: Style::default().front(muted).effect(Effect::Bold),
            header: Style::default().front(bright).effect(Effect::Bold),
            quote: Style::default().front(dim),
            italic: Style::default().effect(Effect::Italic),
            bold: Style::default().effect(Effect::Bold),
            metadata: Style::default().front(dim),
            username: Style::default().effect(Effect::Bold),
            loading_bar: Style::default()
                .front(palette.light_yellow)
                .back(palette.blue),
            ask_hn: Style::default().front(palette.red).effect(Effect::Bold),
            tell_hn: Style::default().front(palette.yellow).effect(Effect::Bold),
            show_hn: Style::default().front(palette.blue).effect(Effect::Bold),
            launch_hn: Style::default().front(palette.green).effect(Effect::Bold),
        };

        Theme {
            palette,
            component_style,
        }
    }
}

type Rgb = (f64, f64, f64);

const BLACK: Rgb = (0.0, 0.0, 0.0);
const WHITE: Rgb = (255.0, 255.0, 255.0);

/// Parses a `#rgb` or `#rrggbb` color
fn parse_rgb(color: &str) -> Option<Rgb> {
    let hex = color.strip_prefix('#')?;
    let channel = |i: usize| {
        let value = match hex.len() {
            3 => u8::from_str_radix(&hex[i..i + 1], 16).map(|v| v * 17),
            6 => u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16),
            _ => return None,
        };
        value.ok().map(f64::from)
    };

    Some((channel(0)?, channel(1)?, channel(2)?))
}

/// Mixes two colors, `amount` being the weight of `to` in the result
fn mix(from: Rgb, to: Rgb, amount: f64) -> String {
    let channel = |from: f64, to: f64| (from * (1.0 - amount) + to * amount).round() as u8;

    format!(
        "#{:02x}{:02x}{:02x}",
        channel(from.0, to.0),
        channel(from.1, to.1),
        channel(from.2, to.2)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A color scheme defining only the semantic colors a theme must define
    const SEMANTIC: &str = r##"
mode = "dark"
background = "#282c34"
foreground = "#abb2bf"
red = "#e06c75"
green = "#98c379"
yellow = "#e5c07b"
blue = "#61afef"
magenta = "#c678dd"
cyan = "#56b6c2"
"##;

    /// A color scheme predating the semantic palette, defining ANSI colors only
    const LEGACY: &str = r##"
color0 = "#1d2021"
color1 = "#fb4934"
color2 = "#b8bb26"
color3 = "#fabd2f"
color4 = "#83a598"
color5 = "#d3869b"
color6 = "#8ec07c"
color7 = "#d5c4a1"
color8 = "#665c54"
color15 = "#fbf1c7"
"##;

    /// `Color` has no `PartialEq` implementation, so compare the colors' representations
    fn repr(color: Color) -> String {
        format!("{color:?}")
    }

    fn hex(color: &str) -> String {
        repr(Color::parse(color))
    }

    // The expected values below are the ones `omarchy-theme-color`, the resolver every
    // other Omarchy consumer goes through, returns for the same color schemes.

    #[test]
    fn derives_the_colors_a_theme_omits() {
        let colors = Colors::from_str(SEMANTIC).unwrap();

        // shades mixed from a base color
        assert_eq!(colors.get("dark_background"), Some("#1e2127"));
        assert_eq!(colors.get("darker_background"), Some("#14161a"));
        assert_eq!(colors.get("bright_red"), Some("#e68991"));
        assert_eq!(colors.get("bright_green"), Some("#adcf94"));
        assert_eq!(colors.get("bright_yellow"), Some("#eacd95"));
        assert_eq!(colors.get("bright_blue"), Some("#81bff2"));
        assert_eq!(colors.get("bright_magenta"), Some("#d193e4"));
        assert_eq!(colors.get("bright_cyan"), Some("#78c5ce"));

        // colors aliased to another color
        assert_eq!(colors.get("orange"), Some("#e5c07b"));
        assert_eq!(colors.get("muted"), Some("#abb2bf"));
        assert_eq!(colors.get("lighter_background"), Some("#282c34"));
        assert_eq!(colors.get("selection_background"), Some("#282c34"));
        assert_eq!(colors.get("selection_foreground"), Some("#abb2bf"));
    }

    #[test]
    fn resolves_a_legacy_ansi_color_scheme() {
        let colors = Colors::from_str(LEGACY).unwrap();

        assert_eq!(colors.get("background"), Some("#1d2021"));
        assert_eq!(colors.get("foreground"), Some("#d5c4a1"));
        assert_eq!(colors.get("red"), Some("#fb4934"));
        assert_eq!(colors.get("cyan"), Some("#8ec07c"));
        assert_eq!(colors.get("muted"), Some("#665c54"));
        assert_eq!(colors.get("bright_foreground"), Some("#fbf1c7"));
        assert_eq!(colors.get("bright_red"), Some("#fc6d5d"));
        assert_eq!(colors.get("selection_background"), Some("#665c54"));
        assert_eq!(colors.get("dark_background"), Some("#161819"));
    }

    #[test]
    fn rejects_a_color_scheme_without_a_background_or_foreground() {
        assert!(Colors::from_str("").is_err());
        assert!(Colors::from_str(r##"background = "#282c34""##).is_err());
        assert!(Colors::from_str(r##"foreground = "#abb2bf""##).is_err());
        assert!(Colors::from_str("background = 42").is_err());
    }

    #[test]
    fn palette_follows_the_terminal_color_mapping() {
        let palette = Colors::from_str(LEGACY).unwrap().into_theme().palette;

        // the ANSI black/white are the theme's background/foreground, as they are
        // for the terminal the application runs in
        assert_eq!(repr(palette.black), hex("#1d2021"));
        assert_eq!(repr(palette.white), hex("#d5c4a1"));
        assert_eq!(repr(palette.light_black), hex("#665c54"));
        assert_eq!(repr(palette.light_white), hex("#fbf1c7"));

        assert_eq!(repr(palette.background), hex("#1d2021"));
        assert_eq!(repr(palette.foreground), hex("#d5c4a1"));
        assert_eq!(repr(palette.selection_background), hex("#665c54"));
        assert_eq!(repr(palette.selection_foreground), hex("#fbf1c7"));

        assert_eq!(repr(palette.red), hex("#fb4934"));
        assert_eq!(repr(palette.light_red), hex("#fc6d5d"));
        assert_eq!(repr(palette.blue), hex("#83a598"));
        assert_eq!(repr(palette.light_blue), hex("#9cb7ad"));
    }

    #[test]
    fn component_styles_use_the_theme_colors() {
        let style = Colors::from_str(LEGACY)
            .unwrap()
            .into_theme()
            .component_style;

        // the title bar uses the theme's orange, `yellow` when the theme has no orange
        assert_eq!(style.title_bar.back.map(repr), Some(hex("#fabd2f")));
        assert_eq!(style.title_bar.front.map(repr), Some(hex("#1d2021")));
        assert_eq!(style.link.front.map(repr), Some(hex("#83a598")));
        assert_eq!(style.metadata.front.map(repr), Some(hex("#665c54")));
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_rgb("#000000"), Some((0.0, 0.0, 0.0)));
        assert_eq!(parse_rgb("#ffffff"), Some(WHITE));
        assert_eq!(parse_rgb("#fff"), Some(WHITE));
        assert_eq!(parse_rgb("#83a598"), Some((131.0, 165.0, 152.0)));
        assert_eq!(parse_rgb("83a598"), None);
        assert_eq!(parse_rgb("rgb(1e1e1e)"), None);
        assert_eq!(parse_rgb("#zzzzzz"), None);
    }
}
