#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate anyhow;

use starlark::eval::Evaluator;
use starlark::environment::{ FrozenModule, Globals, Module };
use starlark::syntax::{ AstModule, Dialect };
use starlark::values::OwnedFrozenValue;

const LAYOUT_SRC: &str = include_str!("layout.star");
const LAYOUT_CONFIG_SYMBOL: &str = "#layout_config";

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

pub struct Layout<'l> {
    pub name: &'l str,
    layout_config: OwnedFrozenValue,
}

impl<'l> Layout<'l> {
    pub fn from_string(layout_name: &'l str, layout_filename: &str, code: &str) -> anyhow::Result<Layout<'l>> {
        // Execute `code` in a module with layout.star pre-imported
        let ast = AstModule::parse(layout_filename, code.to_string(), &Dialect::Extended)?;
        let globals = Globals::extended();
        let module = Module::new();
        module.import_public_symbols(&LAYOUT_LIB);
        // TODO: add support for `load` statements
        let mut eval = Evaluator::new(&module, &globals);
        let layout_config_value = eval.eval_module(ast)?;

        // Extract the type attribute from the result
        let (_, layout_config_type_attr) =
            layout_config_value
                .get_attr("type", eval.heap())
                .expect("Malformed layout: No type attribute.");
        let layout_config_type =
            layout_config_type_attr
                .unpack_str()
                .expect("Malformed layout: Type attribute is not a string.");

        // Verify that the type is "layout"
        if layout_config_type == "layout" {
            // Get a frozen version of the value by putting it into the module with a name that
            // isn't a valid starlark identifier, freezing the module, then pulling it back out
            // as an owned frozen value.
            module.set(LAYOUT_CONFIG_SYMBOL, layout_config_value);
            let frozen_module = module.freeze()?;
            match frozen_module.get(LAYOUT_CONFIG_SYMBOL) {
                Some(layout_config) => {
                    Ok(Layout {
                        name: layout_name,
                        layout_config: layout_config,
                    })
                },
                None => {
                    Err(anyhow!("Failed to extract layout config from frozen module."))
                }
            }
        } else {
            Err(anyhow!("Malformed layout: Unexpected type '{}' for layout config. Should have been 'layout'.", layout_config_type))
        }
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
    background = solid(color = ref(path = "blue")),
    elements = [],
)
        "##;
        Layout::from_string("test-layout", "test.layout", code).unwrap();
    }
}
