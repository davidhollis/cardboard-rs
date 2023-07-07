pub mod fill;
pub mod font;
pub mod only_if;
pub mod stroke;

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub enum PathStyle {
    Stroke(stroke::Stroke),
    Fill(fill::Fill),
    OnlyIf(only_if::OnlyIf),
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub enum TextStyle {
    Font(font::Font),
    OnlyIf(only_if::OnlyIf),
}
