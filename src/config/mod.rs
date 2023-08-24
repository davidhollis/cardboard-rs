pub mod colors;
pub mod sheets;

#[derive(knuffel::Decode)]
pub struct RawConfig {
    #[knuffel(children(name="color"))]
    colors: Vec<colors::ColorDefinition>,
    #[knuffel(children(name="sheet-type"))]
    sheet_types: Vec<sheets::SheetType>,
}
