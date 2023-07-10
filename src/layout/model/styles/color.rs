use std::str::FromStr;

use lazy_static::lazy_static;
use miette::Diagnostic;
use regex::Regex;
use thiserror::Error;

use crate::layout::templates::TemplateAwareString;

lazy_static! {
    static ref RGB_REGEX: Regex = Regex::new(r#"\Argb\(([^,]+),([^,]+),([^,]+)\)\z"#).unwrap();
    static ref RGBA_REGEX: Regex = Regex::new(r#"\Argba\(([^,]+),([^,]+),([^,]+),([^,]+)\)\z"#).unwrap();
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ColorRef {
    Static(Color),
    Named(TemplateAwareString),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Color {
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
}

impl FromStr for ColorRef {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match RGB_REGEX.captures(s).map(|cap| cap.extract()) {
            Some((_, [red, green, blue])) =>
                red
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ColorParseError::InvalidComponent(red.to_string()))
                    .and_then(|r|
                        green
                            .trim()
                            .parse::<u8>()
                            .map_err(|_| ColorParseError::InvalidComponent(green.to_string()))
                            .and_then(|g|
                                blue
                                    .trim()
                                    .parse::<u8>()
                                    .map_err(|_| ColorParseError::InvalidComponent(blue.to_string()))
                                    .map(|b| ColorRef::Static(Color::RGB(r, g, b)))
                            )
                    ),
            None => match RGBA_REGEX.captures(s).map(|cap| cap.extract()) {
                Some((_, [red, green, blue, alpha])) =>
                    red
                        .trim()
                        .parse::<u8>()
                        .map_err(|_| ColorParseError::InvalidComponent(red.to_string()))
                        .and_then(|r|
                            green
                                .trim()
                                .parse::<u8>()
                                .map_err(|_| ColorParseError::InvalidComponent(green.to_string()))
                                .and_then(|g|
                                    blue
                                        .trim()
                                        .parse::<u8>()
                                        .map_err(|_| ColorParseError::InvalidComponent(blue.to_string()))
                                        .and_then(|b|
                                            alpha
                                                .trim()
                                                .parse::<u8>()
                                                .map_err(|_| ColorParseError::InvalidComponent(alpha.to_string()))
                                                .map(|a| ColorRef::Static(Color::RGBA(r, g, b, a)))

                                        )
                                )
                        ),
                None => Ok(ColorRef::Named(TemplateAwareString::new(s.to_string())))
            }
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ColorParseError {
    #[error("error when parsing color: invalid rgba component '{0}' (must be an integer in the range 0...255)")]
    InvalidComponent(String),
}
