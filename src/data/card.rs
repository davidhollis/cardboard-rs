use std::{collections::HashMap, sync::OnceLock};

use handlebars::Context;
use miette::IntoDiagnostic;
use serde::Serialize;

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

    pub fn layout_name(&self) -> Option<String> {
        self.fields.get("layout").map(|layout_name| layout_name.clone())
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

impl<'a> Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.collect_map(self.fields.iter())
    }
}

pub mod loaders {
    use std::{path::Path, collections::HashMap};

    use miette::IntoDiagnostic;

    use super::Card;

    pub fn load_csv<P: AsRef<Path>>(csv_path: P) -> miette::Result<Vec<Card>> {
        let file_name_stem = csv_path.as_ref().file_stem().and_then(|p| p.to_str());
        let mut csv_reader =
            csv::ReaderBuilder::new()
                .flexible(true)
                .trim(csv::Trim::All)
                .from_path(&csv_path)
                .into_diagnostic()?;
        
        Ok(csv_reader
            .deserialize()
            .enumerate()
            .map(|(idx, card_hash)| {
                let mut card_hash: HashMap<String, String> = card_hash.unwrap_or_default();
                let mut card = Card::new(
                    match card_hash.remove("id") {
                        Some(explicit_card_id) => explicit_card_id,
                        None => match file_name_stem {
                            Some(stem) => format!("{}_{:04}", stem, idx),
                            None => format!("{:04}", idx),
                        },
                    }
                );
                card.fields_mut().extend(card_hash);
                card
            })
            .collect()
        )
    }
}
