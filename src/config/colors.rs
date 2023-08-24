use crate::layout::model::styles::color::ColorRef;

#[derive(knuffel::Decode)]
pub struct ColorDefinition {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(argument, str)]
    pub definition: ColorRef,
}
