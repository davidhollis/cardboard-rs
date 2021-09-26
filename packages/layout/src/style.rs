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

pub enum Fill {
    Solid(Color),
}

impl Fill {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Fill> {
        val.validate_type("fill")?;

        let fill_type = val.extract_string("fill", "fill_type")?;
        match fill_type {
            "solid" => Ok(Fill::Solid(
                Color::from_starlark(val.extract_value("fill", "color")?)?
            )),
            _ => Err(
                anyhow!("Unknown fill type {}. Expected one of ['solid'].", fill_type)
            )
        }
    }
}