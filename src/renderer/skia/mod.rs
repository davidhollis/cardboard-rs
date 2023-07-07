use std::{fmt::Debug, fs::File, io::Write};

use miette::{Diagnostic, IntoDiagnostic};
use skia_safe::{Image, EncodedImageFormat, PictureRecorder, Rect, images::{deferred_from_picture, BitDepth}};
use thiserror::Error;

use crate::{data::Card, layout::Layout};

use super::Renderer;

mod drawing;

pub struct SkiaRenderer;

impl SkiaRenderer {
    pub fn new() -> SkiaRenderer {
        SkiaRenderer
    }
}

impl Renderer for SkiaRenderer {
    type SingleCard = Image;
    type Error = miette::Report;

    fn render_single(&self, card: &Card, layout: &Layout) -> Result<Self::SingleCard, Self::Error> {
        let mut recorder = PictureRecorder::new();
        let bounding_rect =
            Rect::new(
                0f32,
                0f32,
                layout.geometry.width as f32,
                layout.geometry.height as f32
            ); 
        let mut canvas = recorder.begin_recording(bounding_rect, None);
        drawing::draw_elements(
            card,
            &layout.elements,
            &mut canvas,
            layout.geometry.width,
            layout.geometry.height,
        )?;
        let finished_picture =
            recorder
                .finish_recording_as_picture(Some(&bounding_rect))
                .ok_or_else(|| SkiaRendererError::GraphicsError(format!("could not convert draw instructions to picture for card with id {}", card.id)))?;
        let image =
            deferred_from_picture(
                finished_picture,
                (layout.geometry.width as i32, layout.geometry.height as i32),
                None,
                None,
                BitDepth::U8,
                None,
                None,
            )
            .ok_or_else(|| SkiaRendererError::GraphicsError(format!("could not rasterize picture for card with id {}", card.id)))?;
        Ok(image)
    }

    fn write_png<P>(&self, card: Self::SingleCard, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<std::path::Path> + Debug {
        let path_str = format!("{:?}", path);
        let mut png_file = File::create(path).into_diagnostic()?;
        let image_data =
            card
                .encode(None, EncodedImageFormat::PNG, Some(100))
                .ok_or_else(|| SkiaRendererError::GraphicsError(format!("could not encode card image as png ({path_str})")))?;
        png_file.write_all(image_data.as_bytes()).into_diagnostic()?;
        png_file.flush().into_diagnostic()?;
        Ok(())
    }

    fn write_single_pdf<P>(&self, _card: Self::SingleCard, _path: P) -> Result<(), Self::Error>
    where
        P: AsRef<std::path::Path> + Debug {
        todo!()
    }

    fn write_deck_pdf<I, P>(&self, _cards: I, _path: P, _page_metrics: crate::layout::printing::PageMetrics) -> Result<(), Self::Error>
    where
        I: Iterator<Item = Self::SingleCard>,
        P: AsRef<std::path::Path> + Debug {
        todo!()
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum SkiaRendererError {
    #[error("encountered an error while rendering a card: {0}")]
    GraphicsError(String),
}