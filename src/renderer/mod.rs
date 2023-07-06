use std::path::Path;

use crate::{data::Card, layout::{Layout, printing::PageMetrics}};

pub trait Renderer {
    type SingleCard;
    type Error;

    fn render_single(&self, card: &Card, layout: &Layout) -> Result<Self::SingleCard, Self::Error>;
    fn write_png<P>(&self, card: Self::SingleCard, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>;
    fn write_single_pdf<P>(&self, card: Self::SingleCard, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<Path>;
    fn write_deck_pdf<I, P>(&self, cards: I, path: P, page_metrics: PageMetrics) -> Result<(), Self::Error>
    where
        I: Iterator<Item = Self::SingleCard>,
        P: AsRef<Path>;
}