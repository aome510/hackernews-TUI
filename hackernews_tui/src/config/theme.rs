use serde::{de, Deserialize, Deserializer};

#[derive(Clone, Debug, Deserialize)]
pub struct Theme {
    pub palette: Palette,
    pub component_style: ComponentStyle,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct ComponentStyle {}

#[derive(Clone, Debug)]
pub struct Color(cursive::theme::Color);

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match cursive::theme::Color::parse(&s) {
            None => Err(de::Error::custom(format!("failed to parse color: {}", s))),
            Some(color) => Ok(Color(color)),
        }
    }
}
