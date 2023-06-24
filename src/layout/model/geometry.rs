#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Geometry {
    #[knuffel(child, unwrap(argument))]
    pub width: usize,
    #[knuffel(child, unwrap(argument))]
    pub height: usize,
    #[knuffel(child)]
    pub cut: Insets,
    #[knuffel(child)]
    pub safe: Insets,
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Insets {
    // TODO: custom knuffel::Decode impl to allow shorthand for uniform insets
    #[knuffel(argument)]
    pub top: usize,
    #[knuffel(argument)]
    pub right: usize,
    #[knuffel(argument)]
    pub bottom: usize,
    #[knuffel(argument)]
    pub left: usize,
}

impl Insets {
    pub fn uniform(size: usize) -> Insets {
        Insets { top: size, right: size, bottom: size, left: size }
    }
}