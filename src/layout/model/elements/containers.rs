use super::Element;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Box {
    #[knuffel(property)]
    pub x: usize,
    #[knuffel(property)]
    pub y: usize,
    #[knuffel(property)]
    pub w: usize,
    #[knuffel(property)]
    pub h: usize,
    #[knuffel(children)]
    pub contents: Vec<Element>,
}
