use miette::Diagnostic;
use thiserror::Error;

use crate::layout::model::styles::color::{ColorRef, Color};

#[derive(knuffel::Decode)]
pub struct ColorDefinition {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument, str)]
    pub definition: ColorRef,
}

impl ColorDefinition {
    pub fn resolve_color(&self) -> Result<Color, ColorDefinitionError> {
        match self.definition {
            ColorRef::Named(_) => Err(ColorDefinitionError::InvalidColorDefinition(self.name.clone())),
            ColorRef::Static(ref color) => Ok(color.clone())
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ColorDefinitionError {
    #[error("invalid color defintion for \"{0}\"--custom colors must be explicit rgb() or rgba() tuples")]
    InvalidColorDefinition(String),
}
