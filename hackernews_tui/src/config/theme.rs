use config_parser2::*;
use cursive::theme::BaseColor;
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

    pub light_black: Color,
    pub light_white: Color,
    pub light_red: Color,
    pub light_magenta: Color,
    pub light_green: Color,
    pub light_cyan: Color,
    pub light_blue: Color,
    pub light_yellow: Color,
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

            black: Color::parse("#000000"),
            blue: Color::parse("#0000aa"),
            cyan: Color::parse("#00aaaa"),
            green: Color::parse("#00aa00"),
            magenta: Color::parse("#aa00aa"),
            red: Color::parse("#aa0000"),
            white: Color::parse("#aaaaaa"),
            yellow: Color::parse("#aaaa00"),

            light_black: Color::parse("#555555"),
            light_white: Color::parse("#ffffff"),
            light_red: Color::parse("#ff5555"),
            light_magenta: Color::parse("#5555ff"),
            light_green: Color::parse("#55ff55"),
            light_cyan: Color::parse("#55ffff"),
            light_blue: Color::parse("#5555ff"),
            light_yellow: Color::parse("#ffff55"),
        }
    }
}

impl Default for ComponentStyle {
    fn default() -> Self {
        Self {
            title_bar: ColorStyle::back(Color::parse("#ff6600")),
            link: ColorStyle::front(Color::parse("#4fbbfd")),
            link_id: ColorStyle::back(Color::parse("light yellow")),
            matched_highlight: ColorStyle::back(Color::parse("light yellow")),
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
            (Some(f), Some(b)) => Self::new(f, b),
            (Some(f), None) => Self::front(f),
            (None, Some(b)) => Self::back(b),
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
        // converts from application's color to `cursive::theme::color` will
        // require to look into the application's pre-defined color palette.
        //
        // Under the hood, the application's palette colors are stored as a wrapper
        // struct of `cursive::theme::color` (`Color`).
        let palette = &get_config_theme().palette;
        match c.0 {
            Self::Dark(c) => match c {
                BaseColor::Black => palette.black.0,
                BaseColor::Red => palette.red.0,
                BaseColor::Green => palette.green.0,
                BaseColor::Yellow => palette.yellow.0,
                BaseColor::Blue => palette.blue.0,
                BaseColor::Magenta => palette.magenta.0,
                BaseColor::Cyan => palette.cyan.0,
                BaseColor::White => palette.white.0,
            },
            Self::Light(c) => match c {
                BaseColor::Black => palette.light_black.0,
                BaseColor::Red => palette.light_red.0,
                BaseColor::Green => palette.light_green.0,
                BaseColor::Yellow => palette.light_yellow.0,
                BaseColor::Blue => palette.light_blue.0,
                BaseColor::Magenta => palette.light_magenta.0,
                BaseColor::Cyan => palette.light_cyan.0,
                BaseColor::White => palette.light_white.0,
            },
            _ => c.0,
        }
    }
}

impl From<Color> for cursive::theme::ColorType {
    fn from(c: Color) -> Self {
        Self::from(cursive::theme::Color::from(c))
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
