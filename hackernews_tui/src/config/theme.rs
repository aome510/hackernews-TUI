use config_parser2::*;
use serde::{de, Deserialize, Deserializer};

#[derive(Default, Clone, Copy, Debug, Deserialize, ConfigParse)]
pub struct Theme {
    pub palette: Palette,
    pub component_style: ComponentStyle,
}

#[derive(Clone, Copy, Debug, Deserialize, ConfigParse)]
pub struct Palette {
    pub background: Color,
    pub foreground: Color,
    pub selection_background: Color,
    pub selection_foreground: Color,

    pub black: Color,
    pub blue: Color,
    pub cyan: Color,
    pub green: Color,
    pub magenta: Color,
    pub red: Color,
    pub white: Color,
    pub yellow: Color,

    pub bright_black: Color,
    pub bright_white: Color,
    pub bright_red: Color,
    pub bright_magenta: Color,
    pub bright_green: Color,
    pub bright_cyan: Color,
    pub bright_blue: Color,
    pub bright_yellow: Color,
}

#[derive(Clone, Copy, Debug, Deserialize, ConfigParse)]
pub struct ComponentStyle {
    pub title_bar: ColorStyle,
    pub link: ColorStyle,
    pub link_id: ColorStyle,
    pub matched_highlight: ColorStyle,
    pub code_block: ColorStyle,
    pub metadata: ColorStyle,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            background: Color::parse("#f6f6ef"),
            foreground: Color::parse("#4a4a48"),
            selection_background: Color::parse("#6c6c6c"),
            selection_foreground: Color::parse("#c3bbbb"),

            black: Color::parse("black"),
            blue: Color::parse("blue"),
            cyan: Color::parse("cyan"),
            green: Color::parse("green"),
            magenta: Color::parse("magenta"),
            red: Color::parse("red"),
            white: Color::parse("white"),
            yellow: Color::parse("yellow"),

            bright_black: Color::parse("light black"),
            bright_white: Color::parse("light white"),
            bright_red: Color::parse("light red"),
            bright_magenta: Color::parse("light magenta"),
            bright_green: Color::parse("light green"),
            bright_cyan: Color::parse("light cyan"),
            bright_blue: Color::parse("light blue"),
            bright_yellow: Color::parse("light yellow"),
        }
    }
}

impl Default for ComponentStyle {
    fn default() -> Self {
        Self {
            title_bar: ColorStyle::back(Color::parse("#ff6600")),
            link: ColorStyle::front(Color::parse("#4fbbfd")),
            link_id: ColorStyle::back(Color::parse("#ffff00")),
            matched_highlight: ColorStyle::back(Color::parse("#ffff00")),
            code_block: ColorStyle::back(Color::parse("#c8c8c8")),
            metadata: ColorStyle::front(Color::parse("#a5a5a5")),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ColorStyle {
    front: Option<Color>,
    back: Option<Color>,
}

config_parser_impl!(ColorStyle);

impl ColorStyle {
    pub fn new(f: Color, b: Color) -> Self {
        Self {
            front: Some(f),
            back: Some(b),
        }
    }
    pub fn front(c: Color) -> Self {
        Self {
            front: Some(c),
            back: None,
        }
    }
    pub fn back(c: Color) -> Self {
        Self {
            front: None,
            back: Some(c),
        }
    }
}

impl From<ColorStyle> for cursive::theme::ColorStyle {
    fn from(c: ColorStyle) -> Self {
        match (c.front, c.back) {
            (Some(f), Some(b)) => Self::new(f.0, b.0),
            (Some(f), None) => Self::front(f.0),
            (None, Some(b)) => Self::back(b.0),
            (None, None) => Self::inherit_parent(),
        }
    }
}

impl From<ColorStyle> for cursive::theme::Style {
    fn from(c: ColorStyle) -> Self {
        Self::from(cursive::theme::ColorStyle::from(c))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color(cursive::theme::Color);

config_parser_impl!(Color);

impl Color {
    pub fn try_parse(c: &str) -> Option<Self> {
        cursive::theme::Color::parse(c).map(Color)
    }

    pub fn parse(c: &str) -> Self {
        Self::try_parse(c).unwrap_or_else(|| panic!("failed to parse color: {}", c))
    }
}

impl From<Color> for cursive::theme::Color {
    fn from(c: Color) -> Self {
        c.0
    }
}

impl From<Color> for cursive::theme::Style {
    fn from(c: Color) -> Self {
        Self::from(cursive::theme::Color::from(c))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match Self::try_parse(&s) {
            None => Err(de::Error::custom(format!("failed to parse color: {}", s))),
            Some(color) => Ok(color),
        }
    }
}

pub fn get_config_theme() -> &'static Theme {
    &super::get_config().theme
}
