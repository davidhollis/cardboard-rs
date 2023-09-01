use oxilangtag::LanguageTag;
use sys_locale::get_locale;

use crate::layout::model::geometry::Geometry;

use self::{sizes::PageSize, placement::{Automatic, automatic::{CropLines, CropLineLength, Margins, Gutter, Align}}};

pub mod layout;
pub mod placement;
pub mod sizes;
pub mod units;

const ASSUMED_DPI: f32 = 300.;

#[derive(knuffel::Decode)]
pub struct SheetType {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(property, str)]
    units: units::Units,
    #[knuffel(child)]
    page_size: sizes::PageSize,
    #[knuffel(child)]
    card_size: sizes::CardSize,
    #[knuffel(child)]
    automatic: Option<placement::Automatic>,
    #[knuffel(child)]
    manual: Option<placement::Manual>,
}

impl SheetType {
    pub fn compile(&self) -> miette::Result<layout::Sheet> {
        let page_size = layout::Dimensions::from(self.page_size.get_dimensions_in_points(&self.units));
        let card_size = layout::Dimensions::from(self.card_size.get_dimensions_in_points(&self.units));

        if let Some(ref manual_placement) = self.manual {
            if self.automatic.is_some() {
                log::warn!("Sheet type \"{}\" has both automatic and manual placement. Ignoring the automatic placement config and keeping the manual placement.", self.name);
            }
            let (cards, crop_lines) = manual_placement.compile(&self.units);
            Ok(layout::Sheet {
                page_size,
                card_size,
                crop_lines,
                cards,
            })
        } else if let Some(ref auto_placement) = self.automatic {
            let (cards, crop_lines) = auto_placement.compile(&page_size, &card_size, &self.units);
            Ok(layout::Sheet {
                page_size,
                card_size,
                crop_lines,
                cards,
            })
        } else {
            Err(placement::PlacementError::Missing.into())
        }
    }

    pub fn sythesize_for(card_geometry: Geometry) -> SheetType {
        // Select the card size based on the content size of a representative
        // card, assuming 300 dpi
        let (card_width_px, card_height_px) = card_geometry.content_size();
        let assumed_card_width_in = card_width_px as f32 / ASSUMED_DPI;
        let assumed_card_height_in = card_height_px as f32 / ASSUMED_DPI;

        // Select the page size based on the detected locale of the user.
        // If they're in a region with majority US Letter, assume that.
        // Otherwise, assume A4.
        let locale_langauge_tag = get_locale().and_then(|l| LanguageTag::parse(l).ok());
        let user_region = locale_langauge_tag.as_ref().and_then(|lt| lt.region());
        let page_size = match user_region {
            // Source: Unicode CLDR version 43
            // cldr-core :: common :: supplemental :: measurementData :: paperSize
            Some("BZ") | Some("CA") | Some("CL") |
            Some("CO") | Some("CR") | Some("GT") |
            Some("MX") | Some("NI") | Some("PA") |
            Some("PH") | Some("PR") | Some("SV") |
            Some("US") | Some("VE") => PageSize::letter_landscape(),
            _ => PageSize::a4_landscape(),
        };

        SheetType {
            name: "__generated".to_string(),
            units: units::Units::Inches,
            page_size,
            card_size: sizes::CardSize::Custom(assumed_card_width_in, assumed_card_height_in),
            automatic: Some(Automatic {
                crop_lines: Some(CropLines { length: CropLineLength::Margin }),
                margins: Margins { top: 0.5, right: 0.5, bottom: 0.5, left: 0.5 },
                gutter: Gutter::default(),
                align: Align::Center,
            }),
            manual: None,
        }
    }
}
