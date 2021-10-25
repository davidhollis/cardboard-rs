use super::starlark_repr::StarlarkContainer;
use super::shape;
use super::style::{ Fill, Stroke };

pub enum Element {
    Shape(shape::Shape, Stroke, Fill),
}

impl Element {
    pub fn from_starlark<'v>(val: StarlarkContainer<'v>) -> anyhow::Result<Element> {
        val.validate_type("element")?;

        let element_type = val.extract_string("element", "element_type")?;
        match element_type {
            "shape" => Ok(
                Element::Shape(
                    shape::Shape::from_starlark(val.extract_value("element.shape", "shape")?)?,
                    Stroke::from_starlark(val.extract_value("element.shape", "stroke")?)?,
                    Fill::from_starlark(val.extract_value("element.shape", "fill")?)?,
                )
            ),
            _ => Err(
                anyhow!("Invalid element type {}. Expected one of ['shape'].", element_type)
            )
        }
    }
}