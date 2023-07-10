use super::color::ColorRef;

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct Solid {
    #[knuffel(argument, str)]
    pub color: ColorRef,
}
