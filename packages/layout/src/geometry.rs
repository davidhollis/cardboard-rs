use super::starlark_repr::StarlarkContainer;

pub struct Insets {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Insets {
    pub fn uniform(length: u16) -> Insets {
        Insets {
            top: length,
            right: length,
            bottom: length,
            left: length,
        }
    }

    pub fn zero() -> Insets { Insets::uniform(0) }

    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Insets> {
        val.validate_type("insets")?;

        Ok(Insets {
            top: val.extract_u16("insets", "top")?,
            right: val.extract_u16("insets", "right")?,
            bottom: val.extract_u16("insets", "bottom")?,
            left: val.extract_u16("insets", "left")?,
        })
    }
}

pub enum Units {
    Pixels,
}

pub struct Geometry {
    pub width: u16,
    pub height: u16,
    pub cut: Insets,
    pub safe: Insets,
    pub units: Units,
}

impl Geometry {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Geometry> {
        val.validate_type("geometry")?;

        Ok(Geometry {
            width: val.extract_u16("geometry", "width")?,
            height: val.extract_u16("geometry", "height")?,
            cut: Insets::from_starlark(val.extract_value("geometry", "cut")?)?,
            safe: Insets::from_starlark(val.extract_value("geometry", "safe")?)?,
            units: Units::Pixels,
        })
    }
}