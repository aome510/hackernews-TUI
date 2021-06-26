use config_parser2::*;
use serde::{de, Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub struct Color {
    pub color: cursive::theme::Color,
}

impl Color {
    fn parse(s: &str) -> Option<Self> {
        cursive::theme::Color::parse(s).map(|color| Color { color })
    }
}

impl<'de> de::Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match Color::parse(&s) {
            None => Err(de::Error::custom(format!("failed to parse color: {}", s))),
            Some(color) => Ok(color),
        }
    }
}

config_parser_impl!(Color);

#[derive(Debug, Deserialize, Clone, ConfigParse)]
pub struct Theme {
    // cursive's palette colors
    pub background: Color,
    pub view: Color,
    pub shadow: Color,
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub title_primary: Color,
    pub title_secondary: Color,
    pub highlight: Color,
    pub highlight_inactive: Color,
    pub highlight_text: Color,

    // additional custom colors
    pub link_text: Color,
    pub link_id_bg: Color,
    pub search_highlight_bg: Color,
    pub status_bar_bg: Color,
    pub code_block_bg: Color,
}

impl Theme {
    pub fn update_theme(&self, theme: &mut cursive::theme::Theme) {
        theme.palette.set_color("background", self.background.color);
        theme.palette.set_color("view", self.view.color);
        theme.palette.set_color("shadow", self.shadow.color);
        theme.palette.set_color("primary", self.primary.color);
        theme.palette.set_color("secondary", self.secondary.color);
        theme.palette.set_color("tertiary", self.tertiary.color);
        theme
            .palette
            .set_color("title_primary", self.title_primary.color);
        theme
            .palette
            .set_color("title_secondary", self.title_secondary.color);
        theme.palette.set_color("highlight", self.highlight.color);
        theme
            .palette
            .set_color("highlight_inactive", self.highlight_inactive.color);
        theme
            .palette
            .set_color("highlight_text", self.highlight_text.color);
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: Color::parse("#f6f6ef").unwrap(),
            shadow: Color::parse("#000000").unwrap(),
            view: Color::parse("#f6f6ef").unwrap(),
            primary: Color::parse("#4a4a48").unwrap(),
            secondary: Color::parse("#a5a5a5").unwrap(),
            tertiary: Color::parse("#ffffff").unwrap(),
            title_primary: Color::parse("#000000").unwrap(),
            title_secondary: Color::parse("#ffff00").unwrap(),
            highlight: Color::parse("#6c6c6c").unwrap(),
            highlight_inactive: Color::parse("#0000ff").unwrap(),
            highlight_text: Color::parse("#c3bbbb").unwrap(),

            link_text: Color::parse("#4fbbfd").unwrap(),
            link_id_bg: Color::parse("#ffff00").unwrap(),
            search_highlight_bg: Color::parse("#ffff00").unwrap(),
            status_bar_bg: Color::parse("#ff6600").unwrap(),
            code_block_bg: Color::parse("#c8c8c8").unwrap(),
        }
    }
}

pub fn get_config_theme() -> &'static Theme {
    &super::get_config().theme
}
