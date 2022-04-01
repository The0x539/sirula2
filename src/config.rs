use std::collections::HashMap;

use gtk4::pango::{self, Attribute};

use directories::ProjectDirs;
use once_cell::sync::Lazy;
use serde::{de::Error, Deserialize, Deserializer};

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Field {
    Comment,
    Id,
    IdSuffix,
    Executable,
    Commandline,
}

#[derive(Debug, Default, Copy, Clone, Deserialize)]
pub struct Sides<T> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T> Sides<T> {
    pub const fn new(left: T, right: T, top: T, bottom: T) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_markup")]
    pub markup_default: Vec<Attribute>,
    #[serde(deserialize_with = "deserialize_markup")]
    pub markup_highlight: Vec<Attribute>,
    #[serde(deserialize_with = "deserialize_markup")]
    pub markup_extra: Vec<Attribute>,
    pub exclusive: bool,
    pub recent_first: bool,
    pub icon_size: i32,
    pub lines: i32,
    pub margin: Sides<i32>,
    pub anchor: Sides<bool>,
    pub width: i32,
    pub height: i32,
    pub extra_field: Vec<Field>,
    pub hidden_fields: Vec<Field>,
    pub name_overrides: HashMap<String, String>,
    pub hide_extra_if_contained: bool,
    pub command_prefix: String,
    pub exclude: Vec<String>,
    pub term_command: Option<String>,
}

impl Config {
    pub fn get() -> &'static Self {
        static CONFIG: Lazy<Config> = Lazy::new(Config::load);
        &CONFIG
    }

    fn load() -> Self {
        let dirs = ProjectDirs::from("", "The0x539", "sirula").unwrap();
        let config_path = dirs.config_dir().join("config.toml");
        let config_str = std::fs::read(config_path).unwrap_or_default();
        toml::from_slice(&config_str).unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            markup_default: vec![],
            markup_highlight: parse_attributes(r#"foreground="red" underline="double""#).unwrap(),
            markup_extra: parse_attributes(r#"font_style="italic" font_size="smaller""#).unwrap(),
            exclusive: true,
            recent_first: true,
            icon_size: 64,
            lines: 2,
            margin: Sides::new(0, 0, 0, 0),
            anchor: Sides::new(false, true, true, true),
            width: -1,
            height: -1,
            extra_field: vec![Field::IdSuffix],
            hidden_fields: vec![],
            name_overrides: HashMap::new(),
            hide_extra_if_contained: true,
            command_prefix: ":".into(),
            exclude: vec![],
            term_command: None,
        }
    }
}

fn deserialize_markup<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<Attribute>, D::Error> {
    let markup = Deserialize::deserialize(de)?;
    parse_attributes(markup).map_err(D::Error::custom)
}

fn parse_attributes(markup: &str) -> Result<Vec<Attribute>, String> {
    let tag = format!("<span {markup}>X</span>");
    let attrs = pango::parse_markup(&tag, '\0')
        .map_err(|e| format!("Failed to parse markup: {e}"))?
        .0
        .iterator()
        .ok_or("Failed to parse markup")?
        .attrs();

    Ok(attrs)
}
