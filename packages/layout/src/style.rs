use super::references::LiteralOrReference;
use super::starlark_repr::StarlarkContainer;

pub enum Color {
    RGBA { red: u16, green: u16, blue: u16, alpha: u16 },
    Named { name: LiteralOrReference },
}

impl Color {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Color> {
        val.validate_type("color")?;

        let color_type = val.extract_string("color", "color_type")?;
        match color_type {
            "rgba" => Ok(Color::RGBA {
                red: val.extract_u16("color", "red")?,
                green: val.extract_u16("color", "green")?,
                blue: val.extract_u16("color", "blue")?,
                alpha: val.extract_u16("color", "alpha")?,
            }),
            "named" => Ok(Color::Named {
                name: LiteralOrReference::from_starlark(val.extract_value("color", "name")?)?,
            }),
            _ => Err(
                anyhow!("Unknown color type {}. Expected one of ['rgba', 'named'].", color_type)
            ),
        }
    }
}

pub enum StrokeType {
    Solid,
    Dashed,
    Dotted,
    None,
}

impl StrokeType {
    pub fn from_string(stroke_type_str: &str) -> StrokeType {
        match stroke_type_str {
            "solid"  => StrokeType::Solid,
            "dashed" => StrokeType::Dashed,
            "dotted" => StrokeType::Dotted,
            _        => StrokeType::None,
        }
    }
}

pub struct Stroke {
    pub stroke_type: StrokeType,
    pub color: Color,
    pub width: u16,
}

impl Stroke {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Stroke> {
        val.validate_type("stroke")?;

        Ok(
            Stroke {
                stroke_type: StrokeType::from_string(val.extract_string("stroke", "type")?),
                color: Color::from_starlark(val.extract_value("stroke", "color")?)?,
                width: val.extract_u16("stroke", "width")?,
            }
        )
    }
}

pub enum Fill {
    Solid(Color),
    None,
}

impl Fill {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Fill> {
        val.validate_type("fill")?;

        let fill_type = val.extract_string("fill", "fill_type")?;
        match fill_type {
            "solid" => Ok(Fill::Solid(
                Color::from_starlark(val.extract_value("fill", "color")?)?
            )),
            "none" => Ok(Fill::None),
            _ => Err(
                anyhow!("Unknown fill type {}. Expected one of ['solid'].", fill_type)
            )
        }
    }
}