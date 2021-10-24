use super::starlark_repr::StarlarkContainer;
use super::shape;
use super::style::{ Fill, Stroke };

pub trait Element: private::Sealed {}

pub struct ElementUtils {}

impl ElementUtils {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Box<dyn Element>> {
        val.validate_type("element")?;

        let element_type = val.extract_string("element", "element_type")?;
        match element_type {
            "shape" => Ok(Box::new(Shape::from_starlark(val)?)),
            _ => Err(
                anyhow!("Invalid element type {}. Expected one of ['shape'].", element_type)
            )
        }
    }
}

pub struct Shape {
    pub shape: shape::Shape,
    pub stroke: Stroke,
    pub fill: Fill,
}

impl Shape {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Shape> {
        Ok(
            Shape {
                shape: shape::Shape::from_starlark(val.extract_value("element.shape", "shape")?)?,
                stroke: Stroke::from_starlark(val.extract_value("element.shape", "stroke")?)?,
                fill: Fill::from_starlark(val.extract_value("element.shape", "fill")?)?,
            }
        )
    }
}

impl Element for Shape {}

mod private {
    pub trait Sealed {}

    impl Sealed for super::Shape {}
}