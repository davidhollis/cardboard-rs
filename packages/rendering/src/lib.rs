pub mod cairo;

use std::path::Path;

#[macro_use]
extern crate anyhow;

pub trait Renderer {
    fn render_png_to(&self, context: Box<dyn content::Context>, path: &Path) -> anyhow::Result<()>;
}