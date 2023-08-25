pub mod containers;
pub mod image;
pub mod shapes;
pub mod text;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub enum Element {
    // TODO: triangle, polygon, star, ellipse, circle, path,
    //       group {transform;clip}
    Rectangle(shapes::Rectangle),
    Text(text::Text),
    Image(image::Image),
    Box(containers::Box),
    Background(shapes::Background),
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Frame {
    #[knuffel(property)]
    pub x: usize,
    #[knuffel(property)]
    pub y: usize,
    #[knuffel(property)]
    pub w: usize,
    #[knuffel(property)]
    pub h: usize,
}

impl Frame {
    pub fn center(&self) -> (usize, usize) {
        (self.x + self.w/2, self.y + self.h/2)
    }
}
