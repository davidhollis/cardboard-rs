use std::{str::FromStr};

use miette::{Diagnostic, SourceOffset};
use thiserror::Error;

use super::color::ColorRef;

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct Stroke {
    // TODO(#9): expand defintion, add joint style
    // https://github.com/davidhollis/cardboard-rs/issues/9
    #[knuffel(argument)]
    pub width: usize,
    #[knuffel(argument, str)]
    pub color: ColorRef,
    #[knuffel(child, unwrap(argument, str), default)]
    pub pattern: DashPattern,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DashPattern {
    Solid,
    Dashed(Vec<u8>),
}

impl Default for DashPattern {
    fn default() -> Self {
        DashPattern::Solid
    }
}

impl FromStr for DashPattern {
    type Err = StrokeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "dotted" {
            return Ok(DashPattern::Dashed(vec![1, 1]))
        }

        if s == "dashed" {
            return Ok(DashPattern::Dashed(vec![3, 1]))
        }

        let mut pattern = vec![];
        let mut reading_dashes = true;
        let mut current_segment_length = 0u8;

        for (idx, ch) in s.char_indices() {
            match ch {
                '.' => {
                    if reading_dashes {
                        current_segment_length += 1;
                    } else {
                        pattern.push(current_segment_length);
                        current_segment_length = 1;
                        reading_dashes = true;
                    }
                },
                '-' => {
                    if reading_dashes {
                        current_segment_length += 3;
                    } else {
                        pattern.push(current_segment_length);
                        current_segment_length = 3;
                        reading_dashes = true;
                    }
                },
                ' ' => {
                    if reading_dashes {
                        pattern.push(current_segment_length);
                        current_segment_length = 1;
                        reading_dashes = false;
                    } else {
                        current_segment_length += 1;
                    }
                },
                _ => return Err(
                    StrokeError::InvalidCharacter {
                        character: ch,
                        dash_spec: s.to_string(),
                        offset: idx.into(),
                    }
                ),
            }
        }

        pattern.push(current_segment_length);

        Ok(DashPattern::Dashed(pattern))
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum StrokeError {
    #[error("unexpected '{character}' in dash pattern")]
    InvalidCharacter {
        character: char,
        #[source_code]
        dash_spec: String,
        #[label("invalid character")]
        offset: SourceOffset,
    },
}

#[cfg(test)]
mod tests {
    use super::DashPattern;

    #[test]
    fn it_recognizes_simple_dotted_pattern() -> miette::Result<()> {
        let test_pattern: DashPattern = "dotted".parse()?;

        assert_eq!(test_pattern, DashPattern::Dashed(vec![1, 1]));

        Ok(())
    }

    #[test]
    fn it_recognizes_simple_dashed_pattern() -> miette::Result<()> {
        let test_pattern: DashPattern = "dashed".parse()?;

        assert_eq!(test_pattern, DashPattern::Dashed(vec![3, 1]));

        Ok(())
    }

    #[test]
    fn it_parses_a_complex_pattern() -> miette::Result<()> {
        let test_pattern: DashPattern = "---   .-.  ... -. .    ".parse()?;

        assert_eq!(test_pattern, DashPattern::Dashed(
            //   ---  [ ][ ][ ]  .-.  [ ][ ]  ...  [ ]  -.  [ ]  .  [ ][ ][ ][ ]
            vec![  9,         3,   5,      2,   3,   1,  4,   1, 1,            4]
        ));

        Ok(())
    }

    #[test]
    fn it_complains_about_invalid_input() -> () {
        let test_pattern = "not a dash pattern".parse::<DashPattern>();

        assert!(test_pattern.is_err())
    }
}
