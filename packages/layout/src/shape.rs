use super::starlark_repr::StarlarkContainer;

pub enum Shape {
    Rectangle(Rectangle)
}

impl Shape {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Shape> {
        val.validate_type("shape")?;

        let shape_type = val.extract_string("shape", "shape_type")?;
        match shape_type {
            "rectangle" => Ok(Shape::Rectangle(Rectangle::from_starlark(val)?)),
            _ => Err(
                anyhow!("Invalid shape type {}. Expected one of ['rectangle'].", shape_type)
            )
        }
    }
}

pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub corner_radius: u16,
}

impl Rectangle {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Rectangle> {
        Ok(
            Rectangle {
                x: val.extract_i16("rectangle", "x")?,
                y: val.extract_i16("rectangle", "y")?,
                width: val.extract_u16("rectangle", "width")?,
                height: val.extract_u16("rectangle", "height")?,
                corner_radius: val.extract_u16("rectangle", "corner_radius").unwrap_or(0),
            }
        )
    }
}