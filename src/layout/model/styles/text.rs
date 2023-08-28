use std::{convert::Infallible, str::FromStr};

use crate::layout::{OnlyIf, Font};

use super::{color::ColorRef, font, TextStyle};

const POINTS_PER_INCH: f32 = 72.0f32;

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct Foreground {
    #[knuffel(argument, str)]
    pub color: ColorRef,
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct Background {
    #[knuffel(argument, str)]
    pub color: ColorRef,
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Size {
    #[knuffel(argument)]
    pub size: usize,
    #[knuffel(argument, str, default=Units::Pixels)]
    pub units: Units,
}

impl Size {
    pub fn pixel_size(&self, dpi: usize) -> f32 {
        match self.units {
            Units::Pixels => self.size as f32,
            Units::Points => {
                let pts = self.size as f32;
                pts * (dpi as f32) / POINTS_PER_INCH
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Units {
    Pixels,
    Points,
}

impl FromStr for Units {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace(" ", "").replace("-", "").as_str() {
            "px" => Ok(Units::Pixels),
            "pt" => Ok(Units::Points),
            _ => Ok(Units::Pixels),
        }
    }
}

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Align {
    #[knuffel(argument, str)]
    pub alignment: Alignment,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
}

impl FromStr for Alignment {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace(" ", "").replace("-", "").as_str() {
            "left" => Ok(Alignment::Left),
            "center" => Ok(Alignment::Center),
            "right" => Ok(Alignment::Right),
            "justify" => Ok(Alignment::Justify),
            _ => Ok(Alignment::Left),
        }
    }
}

#[derive(Clone)]
pub struct FlatTextStyle<'a> {
    pub foreground: Option<&'a Foreground>,
    pub background: Option<&'a Background>,
    pub size: Option<&'a Size>,
    pub align: Alignment,
    pub font_family: Option<&'a str>,
    pub font_weight: font::Weight,
    pub font_width: font::Width,
    pub font_style: Option<&'a str>,
    pub conditions: Vec<&'a OnlyIf>,
}

impl<'a> FlatTextStyle<'a> {
    pub fn apply(&mut self, styles: &'a [TextStyle]) -> () {
        for style in styles {
            match style {
                TextStyle::Font(Font { family, weight, width, style }) => {
                    self.font_family = family.as_ref().map(|s|s.as_str()).or(self.font_family);
                    self.font_style = style.as_ref().map(|s|s.as_str()).or(self.font_style);
                    if let Some(weight) = weight {
                        self.font_weight = *weight;
                    }
                    if let Some(width) = width {
                        self.font_width = *width;
                    }
                },
                TextStyle::Size(sz) => {
                    self.size = Some(sz);
                },
                TextStyle::Align(Align { alignment }) => {
                    self.align = *alignment;
                },
                TextStyle::Foreground(fg) => {
                    self.foreground = Some(fg);
                },
                TextStyle::Background(bg) => {
                    self.background = Some(bg);
                },
                TextStyle::OnlyIf(cond) => {
                    self.conditions.push(cond);
                }
            }
        }
    }
}

impl<'a> Default for FlatTextStyle<'a> {
    fn default() -> Self {
        FlatTextStyle {
            foreground: None,
            background: None,
            size: None,
            align: Alignment::Left,
            font_family: None,
            font_weight: font::Weight::Normal,
            font_width: font::Width::Normal,
            font_style: None,
            conditions: vec![],
        }
    }
}
