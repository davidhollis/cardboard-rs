use std::collections::HashMap;

use crate::layout::model::styles::color::Color;

pub mod colors;
pub mod sheets;
pub mod util;

#[derive(knuffel::Decode)]
pub struct RawConfig {
    #[knuffel(child, unwrap(argument))]
    pub pdf_title: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub pdf_author: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub pdf_subject: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub pdf_keywords: Option<String>,
    #[knuffel(children(name="color"))]
    colors: Vec<colors::ColorDefinition>,
    #[knuffel(children(name="sheet-type"))]
    sheet_types: Vec<sheets::SheetType>,
}

impl RawConfig {
    pub fn get_colors(&self) -> miette::Result<HashMap<String, Color>> {
        let mut color_map = HashMap::new();

        for color in &self.colors {
            color_map.insert(color.name.clone(), color.resolve_color()?);
        }

        Ok(color_map)
    }

    pub fn get_sheet_layouts(&self) -> HashMap<String, sheets::layout::Sheet> {
        let mut sheet_map = HashMap::new();

        for sheet_type in &self.sheet_types {
            sheet_map.insert(sheet_type.name.clone(), sheet_type.compile());
        }

        sheet_map
    }
}
