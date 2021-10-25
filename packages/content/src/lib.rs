use layout::style::Color;

pub trait Context<'a> {
    fn lookup_text(&self, path: &str) -> Option<&'a str>;
    fn lookup_color(&self, path: &str) -> Option<Color>;
}

pub struct EmptyContext {}

impl<'a> Context<'a> for EmptyContext {
    fn lookup_text(&self, _path: &str) -> Option<&'a str> { None }
    fn lookup_color(&self, _path: &str) -> Option<Color> { None }
}