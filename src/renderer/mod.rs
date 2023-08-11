use std::{path::Path, fmt::Debug};

use crate::{data::{project::Project}, layout::{printing::PageMetrics}};

mod skia;
pub use skia::SkiaRenderer;

pub trait Renderer {
    type SingleCard;
    type Error;

    fn render_single(&self, project: &Project, card_id: &str) -> Result<Self::SingleCard, Self::Error>;
    fn write_png<P>(&self, card: Self::SingleCard, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path> + Debug;
    fn write_single_pdf<P>(&self, card: Self::SingleCard, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path> + Debug;
    fn write_deck_pdf<I, P>(&self, cards: I, path: P, page_metrics: PageMetrics) -> Result<(), Self::Error>
    where
        I: Iterator<Item = Self::SingleCard>,
        P: AsRef<Path> + Debug;
}
