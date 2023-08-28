use crate::layout::model::styles::TextStyle as LayoutTextStyle;

#[derive(knuffel::Decode)]
pub struct TextStyle {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children)]
    pub styles: Vec<LayoutTextStyle>,
}
