pub mod containers;
pub mod shapes;
pub mod text;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub enum Element {
    // TODO: triangle, polygon, star, ellipse, circle, path,
    //       group {transform;clip}
    Rectangle(shapes::Rectangle),
    Text(text::Text),
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