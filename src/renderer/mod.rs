use std::{path::Path, fmt::Debug};

use crate::{data::project::Project, config::sheets::layout::Sheet};

mod skia;
pub use skia::SkiaRenderer;

pub trait Renderer {
    type SingleCard<'a>;
    type Error;

    fn render_single<'a>(&mut self, project: &'a Project, card_id: &str) -> Result<Self::SingleCard<'a>, Self::Error>;
    fn write_png<'a, P>(&self, card: &Self::SingleCard<'a>, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path> + Debug;
    fn write_single_pdf<'a, P>(&self, card: &Self::SingleCard<'a>, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path> + Debug;
    fn write_deck_pdf<'a, I, P>(&self, cards: I, path: P, sheet: &Sheet) -> Result<(), Self::Error>
    where
        Self::SingleCard<'a>: 'a,
        I: Iterator<Item = &'a Self::SingleCard<'a>>,
        P: AsRef<Path> + Debug;
}
