use std::{convert::Infallible, str::FromStr};

use super::color::ColorRef;

const POINTS_PER_INCH: f32 = 72.0f32;
const ASSUMED_PIXELS_PER_INCH: f32 = 300.0f32;

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
    pub fn pixel_size(&self) -> f32 {
        match self.units {
            Units::Pixels => self.size as f32,
            Units::Points => {
                let pts = self.size as f32;
                pts * ASSUMED_PIXELS_PER_INCH / POINTS_PER_INCH
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
