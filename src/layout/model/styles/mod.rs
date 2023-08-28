pub mod color;
pub mod font;
pub mod only_if;
pub mod solid;
pub mod stroke;
pub mod text;

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub enum PathStyle {
    Stroke(stroke::Stroke),
    Solid(solid::Solid),
    OnlyIf(only_if::OnlyIf),
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub enum TextStyle {
    Font(font::Font),
    Size(text::Size),
    Align(text::Align),
    Foreground(text::Foreground),
    Background(text::Background),
    OnlyIf(only_if::OnlyIf),
}
