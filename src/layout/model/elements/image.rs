use std::str::FromStr;

use miette::Diagnostic;
use thiserror::Error;

use crate::layout::templates::TemplateAwareString;

use super::Frame;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Image {
    #[knuffel(argument, str)]
    pub name: TemplateAwareString,
    #[knuffel(child)]
    pub frame: Frame,
    #[knuffel(child, unwrap(argument, str))]
    pub scale: Scale,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Scale {
    // Scale the image proportionally so that it's as large as possible while
    // fitting entirely in the frame
    Fit,
    // Scale the image proportionally so that it's as small as possible while
    // filling the whole frame. Any content outside the frame is clipped.
    Fill,
    // Scale the image, potentially stretching it horizontally or vertically,
    // so that it fills the entire frame without being cropped (i.e., so that
    // all 4 of its corners coincide with the corners of the frames).
    Stretch,
    // Do not scale the image. Center it with respect to the frame and clip it
    // to the frame.
    None,
}

impl FromStr for Scale {
    type Err = ImageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fit" => Ok(Scale::Fit),
            "fill" => Ok(Scale::Fill),
            "stretch" => Ok(Scale::Stretch),
            "none" => Ok(Scale::None),
            _ => Err(ImageError::InvalidScaleMode(s.to_string())),
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ImageError {
    #[error("invalid scale mode \"{0}\" (expected one of \"fit\", \"fill\", \"stretch\", or \"none\"")]
    InvalidScaleMode(String)
}
