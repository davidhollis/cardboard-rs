use crate::layout::model::styles::PathStyle;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Rectangle {
    #[knuffel(property)]
    pub x: usize,
    #[knuffel(property)]
    pub y: usize,
    #[knuffel(property)]
    pub w: usize,
    #[knuffel(property)]
    pub h: usize,
    #[knuffel(children)]
    pub style: Vec<PathStyle>,
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Background {
    #[knuffel(children)]
    pub style: Vec<PathStyle>,
}

impl Background {
    pub fn to_rect(&self, frame_width: usize, frame_height: usize) -> Rectangle {
        Rectangle {
            x: 0,
            y: 0,
            w: frame_width,
            h: frame_height,
            style: self.style.clone(),
        }
    }
}
