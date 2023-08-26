use std::{sync::OnceLock, collections::HashMap};

use crate::layout::{model::styles::color::Color, Layout};

static BUILTIN_LAYOUTS: OnceLock<HashMap<&'static str, Layout>> = OnceLock::new();

pub fn init_global_data() -> miette::Result<()> {
    // Explicitly ignore double-initialization, as it should be idempotent
    // TODO(#20): Add some builtin layouts
    // https://github.com/davidhollis/cardboard-rs/issues/20
    let _ = BUILTIN_LAYOUTS.set(HashMap::new());
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
