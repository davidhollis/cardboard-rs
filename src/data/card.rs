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
        match self.fields.get("layout").map(|layout_name| layout_name.clone()) {
            // If there is a layout name, but it's blank, treat that as if there were no layout name
            Some(l) if l == "" => None,
            layout => layout,
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

impl<'a> Serialize for Card {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serializer.collect_map(std::iter::once((&"id".to_string(), &self.id.clone())).chain(self.fields.iter()))
    }
}

pub mod loaders {
    use std::{path::Path, collections::HashMap};

    use calamine::Reader;
    use miette::{IntoDiagnostic, Diagnostic};
    use thiserror::Error;

    use super::Card;

    pub fn load_csv<P: AsRef<Path>>(csv_path: P) -> miette::Result<Vec<Card>> {
        let file_name_stem = csv_path.as_ref().file_stem().and_then(|p| p.to_str()).map(|p| p.to_string());
        let mut csv_reader =
            csv::ReaderBuilder::new()
                .flexible(true)
                .trim(csv::Trim::All)
                .from_path(&csv_path)
                .into_diagnostic()?;
        
        Ok(csv_reader
            .deserialize()
            .map(|hash_result| hash_result.unwrap_or_default())
            .enumerate()
            .map(build_card_from_document_named(file_name_stem))
            .collect()
        )
    }

    pub fn load_excel<P: AsRef<Path>>(excel_path: P) -> miette::Result<Vec<Card>> {
        let file_name_stem = excel_path.as_ref().file_stem().and_then(|p| p.to_str()).map(|p| p.to_string());
        let mut workbook = calamine::open_workbook_auto(&excel_path).into_diagnostic()?;
        let first_sheet = workbook.worksheet_range_at(0).map_or(Err(CardLoadingError::EmptyWorkbook(format!("{}", excel_path.as_ref().display()))).into_diagnostic(), |result| result.into_diagnostic())?;
        let rows = calamine::RangeDeserializerBuilder::new().from_range(&first_sheet).into_diagnostic()?;

        Ok(rows
            .map(|hash_result| hash_result.unwrap_or_default())
            .enumerate()
            .map(build_card_from_document_named(file_name_stem))
            .collect()
        )
    }

    fn build_card_from_document_named(doc_name: Option<String>) -> Box<dyn Fn((usize, HashMap<String, String>)) -> Card> {
        Box::new(move |(idx, mut card_hash)| {
            let mut card = Card::new(
                match card_hash.remove("id") {
                    None => generate_id(idx, &doc_name),
                    // Treat a blank id the same as an absent id
                    Some(blank_id) if blank_id == "" => generate_id(idx, &doc_name),
                    // If there's an explicit id, just use that
                    Some(explicit_card_id) => explicit_card_id,
                }
            );
            card.fields_mut().extend(card_hash);
            card
        })
    }

    fn generate_id(idx: usize, set_name: &Option<String>) -> String {
        match set_name {
            Some(set) => format!("{}_{:04}", set, idx + 1),
            None => format!("{:04}", idx + 1),
        }
    }

    #[derive(Error, Diagnostic, Debug)]
    pub enum CardLoadingError {
        #[error("Spreadsheet workbook {0} conatins no sheets")]
        EmptyWorkbook(String),
    }
}
