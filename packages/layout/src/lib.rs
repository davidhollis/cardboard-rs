#[macro_use]
extern crate lazy_static;

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_loads_the_layout_module() {
        use super::LAYOUT_LIB;
        assert!(LAYOUT_LIB.get("Layout").is_some());
    }
}
