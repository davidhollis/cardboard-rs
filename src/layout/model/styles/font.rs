use std::{str::FromStr, convert::Infallible};

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct Font {
    #[knuffel(property)]
    pub family: Option<String>,
    #[knuffel(property, str)]
    pub weight: Option<Weight>,
    #[knuffel(property, str)]
    pub width: Option<Width>,
    #[knuffel(property)]
    pub style: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Width {
    UltraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiWide,
    Wide,
    UltraWide,
}

impl FromStr for Width {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace(" ", "").replace("-", "").as_str() {
            "ultracondensed" => Ok(Width::UltraCondensed),
            "condensed"      => Ok(Width::Condensed),
            "semicondensed"  => Ok(Width::SemiCondensed),
            "normal"         => Ok(Width::Normal),
            "semiwide"       => Ok(Width::SemiWide),
            "wide"           => Ok(Width::Wide),
            "ultrawide"      => Ok(Width::UltraWide),
            _ => Ok(Width::Normal),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    ExtraBlack,
}

impl FromStr for Weight {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace(" ", "").replace("-", "").as_str() {
            "thin"       => Ok(Weight::Thin),
            "extralight" => Ok(Weight::ExtraLight),
            "light"      => Ok(Weight::Light),
            "normal"     => Ok(Weight::Normal),
            "medium"     => Ok(Weight::Medium),
            "semibold"   => Ok(Weight::SemiBold),
            "bold"       => Ok(Weight::Bold),
            "extrabold"  => Ok(Weight::ExtraBold),
            "black"      => Ok(Weight::Black),
            "extrablack" => Ok(Weight::ExtraBlack),
            _ => Ok(Weight::Normal),
        }
    }
}
