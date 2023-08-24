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
    placement_method: placement::Placement,
}

impl SheetType {
    pub fn compile(&self) -> layout::Sheet {
        let page_size = layout::Dimensions::from(self.page_size.get_dimensions_in_points(&self.units));
        let card_size = layout::Dimensions::from(self.card_size.get_dimensions_in_points(&self.units));

        self.placement_method.compile(page_size, card_size, &self.units)
    }
}
