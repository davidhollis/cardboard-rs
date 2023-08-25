use knuffel::{ast::{Value, Literal}, span::Spanned};
use lazy_static::lazy_static;
use miette::Diagnostic;
use regex::Regex;
use thiserror::Error;

use crate::config::util::extract_number_as_float;

use super::units::Units;

lazy_static! {
    static ref NON_ALNUM_SEQUENCE: Regex = Regex::new(r#"[^A-Za-z0-9]+"#).unwrap();
}

pub struct PageSize {
    dimensions: PageDimensions,
    orientation: Option<PageOrientation>,
}

impl PageSize {
    pub fn get_dimensions_in_points(&self, base_units: &Units) -> (f32, f32) {
        let base_dimenstions = match self.dimensions {
            PageDimensions::Letter => (Units::Inches.convert_to_points(8.5), Units::Inches.convert_to_points(11.)),
            PageDimensions::Legal => (Units::Inches.convert_to_points(8.5), Units::Inches.convert_to_points(14.)),
            PageDimensions::Tabloid => (Units::Inches.convert_to_points(11.), Units::Inches.convert_to_points(17.)),
            PageDimensions::Ledger => (Units::Inches.convert_to_points(17.), Units::Inches.convert_to_points(11.)),
            PageDimensions::A0 => (Units::Millimeters.convert_to_points(841.), Units::Millimeters.convert_to_points(1189.)),
            PageDimensions::A1 => (Units::Millimeters.convert_to_points(594.), Units::Millimeters.convert_to_points(841.)),
            PageDimensions::A2 => (Units::Millimeters.convert_to_points(420.), Units::Millimeters.convert_to_points(594.)),
            PageDimensions::A3 => (Units::Millimeters.convert_to_points(297.), Units::Millimeters.convert_to_points(420.)),
            PageDimensions::A4 => (Units::Millimeters.convert_to_points(210.), Units::Millimeters.convert_to_points(297.)),
            PageDimensions::A5 => (Units::Millimeters.convert_to_points(148.), Units::Millimeters.convert_to_points(210.)),
            PageDimensions::A6 => (Units::Millimeters.convert_to_points(105.), Units::Millimeters.convert_to_points(148.)),
            PageDimensions::A7 => (Units::Millimeters.convert_to_points(74.), Units::Millimeters.convert_to_points(105.)),
            PageDimensions::A8 => (Units::Millimeters.convert_to_points(52.), Units::Millimeters.convert_to_points(74.)),
            PageDimensions::Custom(width, height) => (base_units.convert_to_points(width), base_units.convert_to_points(height)),
        };

        self.orientation.as_ref().map_or(base_dimenstions, |xf| xf.transform_size(base_dimenstions))
    }
}

impl<S> knuffel::Decode<S> for PageSize where S: knuffel::traits::ErrorSpan {
    fn decode_node(node: &knuffel::ast::SpannedNode<S>, _ctx: &mut knuffel::decode::Context<S>)
            -> Result<Self, knuffel::errors::DecodeError<S>> {
        match node.arguments.as_slice() {
            [Value { literal: size_name, .. }] => {
                Ok(PageSize {
                    dimensions: extract_page_size_name(size_name)?,
                    orientation: None,
                })
            },
            [
                Value { literal: size_name, .. },
                Value { literal: orientation_name, .. },
            ] if literal_is_string(size_name) && literal_is_string(orientation_name) => {
                Ok(PageSize {
                    dimensions: extract_page_size_name(size_name)?,
                    orientation: Some(extract_page_orientation_name(orientation_name)?),
                })
            },
            [
                Value { literal: width, .. },
                Value { literal: height, .. },
            ] if literal_is_number(width) && literal_is_number(height) => {
                Ok(PageSize {
                    dimensions: PageDimensions::Custom(
                        extract_number_as_float(width)?,
                        extract_number_as_float(height)?,
                    ),
                    orientation: None,
                })
            },
            _ => Err(knuffel::errors::DecodeError::conversion(node, SizeError::InvalidPageSizeSpecification)),
        }
    }
}

pub enum CardSize {
    Poker,
    Bridge,
    Tarot,
    Square25,
    Square35,
    Mini,
    Custom(f32, f32),
}

impl CardSize {
    pub fn get_dimensions_in_points(&self, base_units: &Units) -> (f32, f32) {
        match self {
            CardSize::Poker => (Units::Millimeters.convert_to_points(63.), Units::Millimeters.convert_to_points(88.)),
            CardSize::Bridge => (Units::Millimeters.convert_to_points(56.), Units::Millimeters.convert_to_points(88.)),
            CardSize::Tarot => (Units::Millimeters.convert_to_points(70.), Units::Millimeters.convert_to_points(121.)),
            CardSize::Square25 => (Units::Millimeters.convert_to_points(63.), Units::Millimeters.convert_to_points(63.)),
            CardSize::Square35 => (Units::Millimeters.convert_to_points(88.), Units::Millimeters.convert_to_points(88.)),
            CardSize::Mini => (Units::Millimeters.convert_to_points(44.), Units::Millimeters.convert_to_points(64.)),
            CardSize::Custom(width, height) => (base_units.convert_to_points(*width), base_units.convert_to_points(*height)),
        }
    }
}

impl<S> knuffel::Decode<S> for CardSize where S: knuffel::traits::ErrorSpan {
    fn decode_node(node: &knuffel::ast::SpannedNode<S>, _ctx: &mut knuffel::decode::Context<S>)
            -> Result<Self, knuffel::errors::DecodeError<S>> {
        match node.arguments.as_slice() {
            [ Value { literal: size_name, .. } ] => {
                Ok(extract_card_size_name(size_name)?)
            },
            [
                Value { literal: width, .. },
                Value { literal: height, .. },
            ] if literal_is_number(width) && literal_is_number(height) => {
                Ok(CardSize::Custom(
                    extract_number_as_float(width)?,
                    extract_number_as_float(height)?,
                ))
            },
            _ => Err(knuffel::errors::DecodeError::conversion(node, SizeError::InvalidCardSizeSpecification)),
        }
    }
}

fn literal_is_string<S>(literal: &Spanned<Literal, S>) -> bool {
    match **literal {
        Literal::String(_) => true,
        _ => false,
    }
}

fn extract_page_size_name<S>(literal: &Spanned<Literal, S>) -> Result<PageDimensions, knuffel::errors::DecodeError<S>>
where S: knuffel::traits::ErrorSpan {
    match **literal {
        Literal::String(ref page_size_name) => match canonicalize_name(&page_size_name).as_str() {
            "letter" | "us letter" => Ok(PageDimensions::Letter),
            "legal" | "us legal" => Ok(PageDimensions::Legal),
            "tabloid" | "us tabloid" => Ok(PageDimensions::Tabloid),
            "ledger" | "us ledger" => Ok(PageDimensions::Ledger),
            "a0" => Ok(PageDimensions::A0),
            "a1" => Ok(PageDimensions::A1),
            "a2" => Ok(PageDimensions::A2),
            "a3" => Ok(PageDimensions::A3),
            "a4" => Ok(PageDimensions::A4),
            "a5" => Ok(PageDimensions::A5),
            "a6" => Ok(PageDimensions::A6),
            "a7" => Ok(PageDimensions::A7),
            "a8" => Ok(PageDimensions::A8),
            _ => Err(knuffel::errors::DecodeError::conversion(literal, SizeError::InvalidPageSizeName(page_size_name.to_string())))
        },
        _ => Err(knuffel::errors::DecodeError::scalar_kind(knuffel::decode::Kind::String, &literal)),
    }
}

fn extract_page_orientation_name<S>(literal: &Spanned<Literal, S>) -> Result<PageOrientation, knuffel::errors::DecodeError<S>>
where S: knuffel::traits::ErrorSpan {
    match **literal {
        Literal::String(ref page_orientation_name) => match canonicalize_name(&page_orientation_name).as_str() {
            "wide" | "landscape" => Ok(PageOrientation::Wide),
            "tall" | "portrait" => Ok(PageOrientation::Tall),
            _ => Err(knuffel::errors::DecodeError::conversion(literal, SizeError::InvalidOrientationName(page_orientation_name.to_string())))
        },
        _ => Err(knuffel::errors::DecodeError::scalar_kind(knuffel::decode::Kind::String, &literal)),
    }
}

fn extract_card_size_name<S>(literal: &Spanned<Literal, S>) -> Result<CardSize, knuffel::errors::DecodeError<S>>
where S: knuffel::traits::ErrorSpan {
    match **literal {
        Literal::String(ref card_size_name) => match canonicalize_name(&card_size_name).as_str() {
            "poker" => Ok(CardSize::Poker),
            "bridge" => Ok(CardSize::Bridge),
            "tarot" => Ok(CardSize::Tarot),
            "mini" => Ok(CardSize::Mini),
            "small square" => Ok(CardSize::Square25),
            "large square" => Ok(CardSize::Square35),
            _ => Err(knuffel::errors::DecodeError::conversion(literal, SizeError::InvalidCardSizeName(card_size_name.to_string())))
        },
        _ => Err(knuffel::errors::DecodeError::scalar_kind(knuffel::decode::Kind::String, &literal)),
    }
}

fn canonicalize_name(name: &str) -> String {
    NON_ALNUM_SEQUENCE.replace_all(name, " ").to_ascii_lowercase()
}

fn literal_is_number<S>(literal: &Spanned<Literal, S>) -> bool {
    match **literal {
        Literal::Int(_) | Literal::Decimal(_) => true,
        _ => false,
    }
}

enum PageDimensions {
    Letter,
    Legal,
    Tabloid,
    Ledger,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    Custom(f32, f32)
}

enum PageOrientation {
    Tall,
    Wide,
}

impl PageOrientation {
    fn transform_size(&self, size: (f32, f32)) -> (f32, f32) {
        let (width, height) = size;
        match self {
            PageOrientation::Tall => {
                // If it's wider than it's tall, flip it to be taller
                if width > height { (height, width) } else { size }
            },
            PageOrientation::Wide => {
                // If it's taller than it's wide, flip it to be wider
                if height > width { (height, width) } else { size }
            }
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
enum SizeError {
    #[error("invalid page size name \"{0}\" (perhaps you meant \"US Letter\" or \"A4\"?)")]
    InvalidPageSizeName(String),
    #[error("invalid orientation name \"{0}\" (perhaps you meant \"portrait\" or \"landscape\"?)")]
    InvalidOrientationName(String),
    #[error("invalid card size name \"{0}\"")]
    InvalidCardSizeName(String),
    #[error("invalid page-size arguments (expected either a name and optional orientation, or an explicit width and height)")]
    InvalidPageSizeSpecification,
    #[error("invalid card-size arguments (expected eitehr a name or an explicit width and height")]
    InvalidCardSizeSpecification,
}
