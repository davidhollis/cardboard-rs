use miette::Diagnostic;
use thiserror::Error;

use super::{layout, units::Units};

#[derive(knuffel::Decode)]
pub struct Automatic {
    #[knuffel(child)]
    crop_lines: Option<automatic::CropLines>,
    #[knuffel(child)]
    margins: automatic::Margins,
    #[knuffel(child, default)]
    gutter: automatic::Gutter,
    #[knuffel(child, unwrap(argument, str, default), default)]
    align: automatic::Align,
}

impl Automatic {
    pub(super) fn compile(&self, page: &layout::Dimensions, card: &layout::Dimensions, base_units: &Units) -> (Vec<layout::CardPlacement>, Vec<layout::CropLine>) {
        let margins = self.margins.into_points(base_units);
        let gutter = self.gutter.into_points(base_units);

        let content = layout::Dimensions {
            width: (page.width - margins.left - margins.right).max(0.),
            height: (page.height - margins.top - margins.bottom).max(0.),
        };
        let ncolumns = ((content.width + gutter.horizontal)/(card.width + gutter.horizontal)).floor() as usize;
        let nrows = ((content.height + gutter.vertical)/(card.height + gutter.vertical)).floor() as usize;

        if ncolumns == 0 || nrows == 0 {
            return (vec![], vec![]);
        }

        let leftover_width = content.width - ((ncolumns as f32) * card.width + ((ncolumns - 1) as f32) * gutter.horizontal);
        let x_start = margins.left + match self.align {
            automatic::Align::Left => 0.,
            automatic::Align::Center => leftover_width / 2.,
            automatic::Align::Right => leftover_width,
        };
        let y_start = margins.top;

        let mut card_slots = vec![];
        let mut crop_lines = vec![];

        let vertical_crop_line_length = match self.crop_lines {
            Some(automatic::CropLines { length: automatic::CropLineLength::Full }) => Some(page.height),
            Some(automatic::CropLines { length: automatic::CropLineLength::Margin }) => Some(margins.top.min(margins.bottom)),
            None => None,
        };
        let horizontal_crop_line_length = match self.crop_lines {
            Some(automatic::CropLines { length: automatic::CropLineLength::Full }) => Some(page.width),
            Some(automatic::CropLines { length: automatic::CropLineLength::Margin }) => Some(margins.left.min(margins.right)),
            None => None,
        };

        for i in 0..ncolumns {
            let column_x = (card.width + gutter.horizontal) * (i as f32) + x_start;
            for j in 0..nrows {
                card_slots.push(layout::CardPlacement {
                    x: column_x,
                    y: (card.height + gutter.vertical) * (j as f32) + y_start,
                    rotate: None,
                    reflect: None,
                })
            }

            if let Some(crop_line_length) = vertical_crop_line_length {
                crop_lines.push(layout::CropLine {
                    orientation: layout::CropLineOrientation::Vertical,
                    offset: column_x,
                    length: crop_line_length,
                });

                if gutter.horizontal > 0. || i == ncolumns - 1 {
                    crop_lines.push(layout::CropLine {
                        orientation: layout::CropLineOrientation::Vertical,
                        offset: column_x + card.width,
                        length: crop_line_length,
                    });
                }
            }
        }

        if let Some(crop_line_length) = horizontal_crop_line_length {
            for j in 0..nrows {
                let row_y = (card.height + gutter.vertical) * (j as f32) + y_start;
                crop_lines.push(layout::CropLine {
                    orientation: layout::CropLineOrientation::Horizontal,
                    offset: row_y,
                    length: crop_line_length,
                });

                if gutter.vertical > 0. || j == nrows - 1 {
                    crop_lines.push(layout::CropLine {
                        orientation: layout::CropLineOrientation::Horizontal,
                        offset: row_y + card.height,
                        length: crop_line_length,
                    });
                }
            }
        }

        (card_slots, crop_lines)
    }
}

mod automatic {
    use std::{str::FromStr, convert::Infallible};

    use knuffel::ast::Value;

    use crate::config::{util::extract_number_as_float, sheets::units::Units};

    use super::PlacementError;

    #[derive(knuffel::Decode)]
    pub(super) struct CropLines {
        #[knuffel(argument, str)]
        pub(super) length: CropLineLength,
    }

    pub(super) enum CropLineLength {
        Margin,
        Full,
    }

    impl FromStr for CropLineLength {
        type Err = Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_ascii_lowercase().as_str() {
                "margin" => Ok(CropLineLength::Margin),
                "full" => Ok(CropLineLength::Full),
                _ => Ok(CropLineLength::Margin),
            }
        }
    }

    pub(super) struct Margins {
        pub(super) top: f32,
        pub(super) right: f32,
        pub(super) bottom: f32,
        pub(super) left: f32,
    }

    impl Margins {
        pub(super) fn into_points(&self, base_units: &Units) -> Margins {
            Margins {
                top: base_units.convert_to_points(self.top),
                right: base_units.convert_to_points(self.right),
                bottom: base_units.convert_to_points(self.bottom),
                left: base_units.convert_to_points(self.left),
            }
        }
    }

    impl<S> knuffel::Decode<S> for Margins
    where S: knuffel::traits::ErrorSpan {
        fn decode_node(node: &knuffel::ast::SpannedNode<S>, _ctx: &mut knuffel::decode::Context<S>)
                -> Result<Self, knuffel::errors::DecodeError<S>> {
            match node.arguments.as_slice() {
                [ Value { literal: length, .. } ] => {
                    let length = extract_number_as_float(length)?;
                    Ok(Margins {
                        top: length,
                        right: length,
                        bottom: length,
                        left: length,
                    })
                },
                [
                    Value { literal: vertical, .. },
                    Value { literal: horizontal, .. },
                ] => {
                    let vertical = extract_number_as_float(vertical)?;
                    let horizontal = extract_number_as_float(horizontal)?;
                    Ok(Margins {
                        top: vertical,
                        right: horizontal,
                        bottom: vertical,
                        left: horizontal,
                    })
                },
                [
                    Value { literal: top, .. },
                    Value { literal: right, .. },
                    Value { literal: bottom, .. },
                    Value { literal: left, .. },
                ] => {
                    Ok(Margins {
                        top: extract_number_as_float(top)?,
                        right: extract_number_as_float(right)?,
                        bottom: extract_number_as_float(bottom)?,
                        left: extract_number_as_float(left)?,
                    })
                },
                _ => Err(knuffel::errors::DecodeError::conversion(node, "invalid number of arguments for margins (expected 1, 2, or 4)")),
            }
        }
    }

    pub(super) struct Gutter {
        pub(super) horizontal: f32,
        pub(super) vertical: f32,
    }

    impl Gutter {
        pub(super) fn into_points(&self, base_units: &Units) -> Gutter {
            Gutter {
                horizontal: base_units.convert_to_points(self.horizontal),
                vertical: base_units.convert_to_points(self.vertical),
            }
        }
    }

    impl<S> knuffel::Decode<S> for Gutter
    where S: knuffel::traits::ErrorSpan {
        fn decode_node(node: &knuffel::ast::SpannedNode<S>, _ctx: &mut knuffel::decode::Context<S>)
                -> Result<Self, knuffel::errors::DecodeError<S>> {
            match node.arguments.as_slice() {
                [ Value { literal: length, .. } ] => {
                    let length = extract_number_as_float(length)?;
                    Ok(Gutter {
                        horizontal: length,
                        vertical: length,
                    })
                },
                [
                    Value { literal: vertical, .. },
                    Value { literal: horizontal, .. },
                ] => {
                    Ok(Gutter {
                        horizontal: extract_number_as_float(horizontal)?,
                        vertical: extract_number_as_float(vertical)?,
                    })
                },
                _ => Err(knuffel::errors::DecodeError::conversion(node, "invalid number of arguments for gutter (expected 1 or 2)")),
            }
        }
    }

    impl Default for Gutter {
        fn default() -> Self {
            Gutter { horizontal: 0., vertical: 0. }
        }
    }

    pub(super) enum Align {
        Left,
        Center,
        Right,
    }

    impl FromStr for Align {
        type Err = PlacementError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "left" => Ok(Align::Left),
                "center" => Ok(Align::Center),
                "right" => Ok(Align::Right),
                _ => Err(PlacementError::InvalidAlignment(s.to_string())),
            }
        }
    }

    impl Default for Align {
        fn default() -> Self {
            Align::Left
        }
    }
}

#[derive(knuffel::Decode)]
pub struct Manual {
    #[knuffel(child)]
    crop_lines: Option<manual::CropLines>,
    #[knuffel(children)]
    cards: Vec<manual::Card>,
}

impl Manual {
    pub(super) fn compile(&self, base_units: &Units) -> (Vec<layout::CardPlacement>, Vec<layout::CropLine>) {
        (
            self.cards.iter().map(|card| card.compile(base_units)).collect(),
            self.crop_lines.as_ref().map(|lines| lines.compile(base_units)).unwrap_or_default(),
        )
    }
}

mod manual {
    use std::str::FromStr;

    use crate::config::sheets::{units::Units, layout::{CardPlacement, self}};

    use super::PlacementError;

    #[derive(knuffel::Decode)]
    pub(super) struct CropLines {
        #[knuffel(argument)]
        pub(super) length: Option<f32>,
        #[knuffel(children)]
        pub(super) lines: Vec<Line>,
    }

    impl CropLines {
        pub(super) fn compile(&self, base_units: &Units) -> Vec<layout::CropLine> {
            self.lines.iter().map(|line| match line {
                Line::Horizontal(offset, length) => layout::CropLine {
                    orientation: layout::CropLineOrientation::Horizontal,
                    offset: base_units.convert_to_points(*offset),
                    length: base_units.convert_to_points(length.or(self.length).unwrap_or(0.)),
                },
                Line::Vertical(offset, length) => layout::CropLine {
                    orientation: layout::CropLineOrientation::Vertical,
                    offset: base_units.convert_to_points(*offset),
                    length: base_units.convert_to_points(length.or(self.length).unwrap_or(0.)),
                },
            }).filter(|line| line.length > 0.).collect()
        }
    }

    #[derive(knuffel::Decode)]
    pub(super) enum Line {
        Horizontal(#[knuffel(property(name="y"))] f32, #[knuffel(property(name="length"))] Option<f32>),
        Vertical(#[knuffel(property(name="x"))] f32, #[knuffel(property(name="length"))] Option<f32>),
    }

    #[derive(knuffel::Decode)]
    pub(super) struct Card {
        #[knuffel(property)]
        pub(super) x: f32,
        #[knuffel(property)]
        pub(super) y: f32,
        #[knuffel(child, unwrap(argument))]
        pub(super) rotate: Option<f32>,
        #[knuffel(child, unwrap(argument, str))]
        pub(super) flip: Option<Axis>,
    }

    impl Card {
        pub(super) fn compile(&self, base_units: &Units) -> CardPlacement {
            CardPlacement {
                x: base_units.convert_to_points(self.x),
                y: base_units.convert_to_points(self.y),
                rotate: self.rotate,
                reflect: match self.flip {
                    Some(Axis::Horizontal) => Some(layout::Axis::Horizontal),
                    Some(Axis::Vertical) => Some(layout::Axis::Vertical),
                    None => None,
                }
            }
        }
    }

    pub(super) enum Axis {
        Horizontal,
        Vertical,
    }

    impl FromStr for Axis {
        type Err = PlacementError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "horizontal" => Ok(Axis::Horizontal),
                "vertical" => Ok(Axis::Vertical),
                _ => Err(PlacementError::InvalidAxis(s.to_string())),
            }
        }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum PlacementError {
    #[error("invalid alignment \"{0}\" (expected one of \"left\", \"right\", or \"center\")")]
    InvalidAlignment(String),
    #[error("invalid reflection axis \"{0}\" (expected one of \"horizontal\" or \"vertical\")")]
    InvalidAxis(String),
    #[error("no placement method specified (expected either an automatic or a manual stanza)")]
    Missing,
}
