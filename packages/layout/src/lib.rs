pub mod element;
pub mod geometry;
pub mod references;
pub mod shape;
pub mod starlark_repr;
pub mod style;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

use starlark::eval::Evaluator;
use starlark::environment::{ FrozenModule, Globals, Module };
use starlark::syntax::{ AstModule, Dialect };

const LAYOUT_SRC: &str = include_str!("layout.star");

lazy_static! {
    static ref LAYOUT_LIB: FrozenModule = {
        let ast: AstModule =
            AstModule::parse("layout.star", LAYOUT_SRC.to_string(), &Dialect::Extended)
            .expect("Parse error in layout.star");
        let globals: Globals = Globals::extended();
        let layout_lib_module: Module = Module::new();
        let mut eval: Evaluator = Evaluator::new(&layout_lib_module, &globals);
        let _ = eval.eval_module(ast).expect("Error evaluating layout.star");
        layout_lib_module.freeze().expect("Failed to freeze layout module")
    };
}

pub struct Layout {
    pub geometry: geometry::Geometry,
    pub background: style::Fill,
    pub elements: Vec<element::Element>,
}

impl Layout {
    pub fn from_starlark<'v>(val: starlark_repr::StarlarkContainer<'v>) -> anyhow::Result<Layout> {
        val.validate_type("layout")?;

        Ok(Layout {
            geometry: geometry::Geometry::from_starlark(
                val.extract_value("layout", "geometry")?
            )?,
            background: style::Fill::from_starlark(
                val.extract_value("layout", "background")?
            )?,
            elements:
                val
                    .extract_value("layout", "elements")?
                    .collect(|v| element::Element::from_starlark(v))?,
        })
    }

    pub fn from_string(layout_filename: &str, code: &str) -> anyhow::Result<Layout> {
        // Execute `code` in a module with layout.star pre-imported
        let ast = AstModule::parse(layout_filename, code.to_string(), &Dialect::Extended)?;
        let globals = Globals::extended();
        let module = Module::new();
        module.import_public_symbols(&LAYOUT_LIB);
        // TODO: add support for `load` statements
        let mut eval = Evaluator::new(&module, &globals);
        let layout_config_value = eval.eval_module(ast)?;

        Ok(
            Layout::from_starlark(
                starlark_repr::StarlarkContainer::new(layout_config_value, eval.heap())
            )?
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_loads_the_layout_module() {
        use super::LAYOUT_LIB;
        assert!(LAYOUT_LIB.get("layout").is_some());
    }

    #[test]
    fn it_constructs_layouts() {
        use super::Layout;
        let code = r##"
layout(
    geometry = geometry(
        width = 100,
        height = 100,
        units = "px",
    ),
    background = solid(color = named("blue")),
    elements = [
        shape(
            rectangle(40, 40, 20, 20),
            stroke = stroke("dotted", color = named("red"), width = 3),
            fill = solid(named("purple")),
        )
    ],
)
        "##;
        Layout::from_string("test.layout", code).unwrap();
    }
}

pub mod prelude {
    pub use crate::Layout;
    pub use crate::element::*;
    pub use crate::geometry::*;
    pub use crate::references::*;
    pub use crate::shape::*;
    pub use crate::style::*;
}
