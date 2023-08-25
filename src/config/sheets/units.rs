use std::str::FromStr;

use miette::Diagnostic;
use thiserror::Error;

const POINTS_PER_INCH: f32 = 72.0;
const POINTS_PER_MILLIMETER: f32 = 2.835;

pub enum Units {
    Points,
    Inches,
    Millimeters,
}

impl Units {
    pub fn convert_to_points(&self, length: f32) -> f32 {
        match self {
            Units::Points => length,
            Units::Inches => length * POINTS_PER_INCH,
            Units::Millimeters => length * POINTS_PER_MILLIMETER,
        }
    }
}

impl FromStr for Units {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase = s.to_ascii_lowercase();
        match lowercase.as_str() {
            "in" => Ok(Units::Inches),
            "pt" => Ok(Units::Points),
            "mm" => Ok(Units::Millimeters),
            _ => Err(UnitError::InvalidUnits(lowercase))
        }
    }
}

pub fn pixels_to_points(length: usize, dpi: usize) -> f32 {
    let length_inches = (length as f32)/(dpi as f32);
    Units::Inches.convert_to_points(length_inches)
}

pub fn scale_factor_at_dpi(dpi: usize) -> f32 {
    POINTS_PER_INCH / (dpi as f32)
}

#[derive(Error, Diagnostic, Debug)]
pub enum UnitError {
    #[error("invalid page unit \"{0}\". Expected one of \"in\", \"pt\", or \"mm\"")]
    InvalidUnits(String),
}
