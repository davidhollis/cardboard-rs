use crate::layout::{model::styles::TextStyle, templates::TemplateAwareString};

use super::Frame;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Text {
    #[knuffel(argument, str)]
    pub contents: TemplateAwareString,
    #[knuffel(child)]
    pub frame: Frame,
    #[knuffel(child, unwrap(argument))]
    pub style: Option<String>,
    #[knuffel(children)]
    pub inline_styles: Vec<TextStyle>,
}
