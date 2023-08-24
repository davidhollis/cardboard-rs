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
    //placement_method
}
