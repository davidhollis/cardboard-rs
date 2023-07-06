use std::{collections::HashMap, sync::OnceLock};

use handlebars::Context;
use serde::Serialize;

pub struct Card {
    fields: HashMap<String, String>,
    handlebars_context: OnceLock<Result<Context, handlebars::RenderError>>,
}

impl<'a> TryFrom<&'a Card> for &'a Context {
    type Error = &'a handlebars::RenderError;

    fn try_from(card: &'a Card) -> Result<Self, Self::Error> {
        card.handlebars_context.get_or_init(|| {
            Context::wraps(card)
        }).as_ref()
    }
}

impl Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.collect_map(self.fields.iter())
    }
}