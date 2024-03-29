use std::{sync::OnceLock, collections::HashMap};

use crate::layout::{model::styles::{color::{Color, ColorRef}, TextStyle, font::{Font, Weight}, text::Foreground}, model::Layout, templates::TemplateAwareString};

static BUILTIN_LAYOUTS: OnceLock<HashMap<&'static str, Layout>> = OnceLock::new();
static BUILTIN_STYLES: OnceLock<HashMap<&'static str, Vec<TextStyle>>> = OnceLock::new();

const DEFAULT_LAYOUT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/layout/builtin/default.layout"));
const ICONS_LAYOUT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/layout/builtin/icons.layout"));

pub fn init_global_data() -> miette::Result<()> {
    let mut layout_map = HashMap::new();
    layout_map.insert("default", knuffel::parse("default.layout", DEFAULT_LAYOUT)?);
    layout_map.insert("icons", knuffel::parse("icons.layout", ICONS_LAYOUT)?);
    // Explicitly ignore double-initialization, as it should be idempotent
    let _ = BUILTIN_LAYOUTS.set(layout_map);

    let mut style_map = HashMap::new();
    style_map.insert("b", vec![
        TextStyle::Font(Font {
            family: None,
            weight: Some(Weight::Bold),
            width: None,
            style: None,
        }),
    ]);
    style_map.insert("i", vec![
        TextStyle::Font(Font {
            family: None,
            weight: None,
            width: None,
            style: Some("italic".to_string()),
        }),
    ]);
    for color in ["black", "dark-gray", "gray", "light-gray", "white", "red", "green", "blue", "yellow", "cyan", "magenta"] {
        style_map.insert(color, vec![
            TextStyle::Foreground(Foreground {
                color: ColorRef::Named(TemplateAwareString::RawString(color.replace("-", " "))),
            }),
        ]);
    }
    let _ = BUILTIN_STYLES.set(style_map);

    Ok(())
}

pub fn layout_named(name: &str) -> Option<&'static Layout> {
    BUILTIN_LAYOUTS.get()
        .and_then(|layouts| layouts.get(name))
}

pub fn color_named(name: &str) -> Option<Color> {
    match name {
        "transparent" => Some(Color::RGBA(0x00, 0x00, 0x00, 0x00)),
        "black" => Some(Color::RGBA(0x00, 0x00, 0x00, 0xff)),
        "dark gray" => Some(Color::RGBA(0x44, 0x44, 0x44, 0xff)),
        "gray" => Some(Color::RGBA(0x88, 0x88, 0x88, 0xff)),
        "light gray" => Some(Color::RGBA(0xcc, 0xcc, 0xcc, 0xff)),
        "white" => Some(Color::RGBA(0xff, 0xff, 0xff, 0xff)),
        "red" => Some(Color::RGBA(0xff, 0x00, 0x00, 0xff)),
        "green" => Some(Color::RGBA(0x00, 0xff, 0x00, 0xff)),
        "blue" => Some(Color::RGBA(0x00, 0x00, 0xff, 0xff)),
        "yellow" => Some(Color::RGBA(0xff, 0xff, 0x00, 0xff)),
        "cyan" => Some(Color::RGBA(0x00, 0xff, 0xff, 0xff)),
        "magenta" => Some(Color::RGBA(0xff, 0x00, 0xff, 0xff)),
        _ => None,
    }
}

pub fn style_named(name: &str) -> Option<&'static [TextStyle]> {
    BUILTIN_STYLES.get()
        .and_then(|styles| styles.get(name))
        .map(|style| style.as_slice())
}
