pub mod layout;
pub mod placement;
pub mod sizes;
pub mod units;

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
}
