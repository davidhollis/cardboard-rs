use std::{collections::HashMap, sync::OnceLock};

use handlebars::Context;
use miette::IntoDiagnostic;
use serde::Serialize;

use crate::layout::model::styles::color::Color;

pub struct Card {
    pub id: String,
    fields: HashMap<String, String>,
    handlebars_context: OnceLock<Context>,
}

impl Card {
    pub fn new(id: String) -> Card {
        Card { id, fields: HashMap::new(), handlebars_context: OnceLock::new() }
    }

    pub fn fields_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.fields
    }

    pub fn color_named(&self, color_name: &str) -> Option<Color> {
        match color_name {
            // TODO: find a better place to store builtin colors
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
            _ => None, // TODO: actually try to look this up in project state
        }
    }
}

impl<'a> TryFrom<&'a Card> for &'a Context {
    type Error = miette::Error;

    fn try_from(card: &'a Card) -> Result<Self, Self::Error> {
        // Doing multiple checks like this may cause multiple threads to
        // concurrently do the same work converting the Card to a context,
        // but that shouldn't yield incorrect results, it's just a bit
        // inefficient.
        //
        // Storing the result itself in the lock doesn't work because we
        // want the error to be able to outlive the Card reference ('a). In
        // fact, in an ideal case we want to return an owned error struct
        // rather than a reference, but everything we get out of a OnceLock
        // is necessarily behind a reference bound to the lifetime of the
        // lock itself.
        //
        // This can be fixed when https://github.com/rust-lang/rust/issues/109737
        // lands and OnceLock::get_or_try_init becomes available.
        match card.handlebars_context.get() {
            Some(ctx) => Ok(ctx),
            None =>
                Context::wraps(card)
                    .map(|new_ctx|
                        card.handlebars_context.get_or_init(move || new_ctx)
                    )
                    .into_diagnostic()
        }
    }
}

impl Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.collect_map(self.fields.iter())
    }
}
