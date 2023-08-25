use std::{fmt::Debug, fs::File, io::Write};

use miette::{Diagnostic, IntoDiagnostic};
use skia_safe::{EncodedImageFormat, PictureRecorder, Rect, Picture, Surface};
use thiserror::Error;

use crate::{data::{project::Project}, config::sheets::{units, layout::Sheet}, layout::Geometry};

use super::Renderer;

mod drawing;
mod pdf;

pub struct SkiaRenderer;

impl SkiaRenderer {
    pub fn new() -> SkiaRenderer {
        SkiaRenderer
    }
}

impl Renderer for SkiaRenderer {
    type SingleCard<'a> = SkiaCard<'a>;
    type Error = miette::Report;

    fn render_single<'a>(&self, project: &'a Project, card_id: &str) -> Result<Self::SingleCard<'a>, Self::Error> {
        let card = project.card_by_id(card_id)?;
        let layout = project.layout_for_card(card)?;

        // Prepare a picture recorder for the card
        let mut recorder = PictureRecorder::new();
        let bounding_rect =
            Rect::new(
                0f32,
                0f32,
                layout.geometry.width as f32,
                layout.geometry.height as f32
            ); 
        let mut canvas = recorder.begin_recording(bounding_rect, None);

        // Draw the card
        let render_ctx = drawing::CardRenderContext::new(card, project, layout.geometry.dpi);
        render_ctx.draw_elements(
            &mut canvas,
            &layout.elements,
            layout.geometry.width,
            layout.geometry.height,
        )?;

        // Finalize and return the picture
        let finished_picture =
            recorder
                .finish_recording_as_picture(Some(&bounding_rect))
                .ok_or_else(|| SkiaRendererError::GraphicsError(format!("could not convert draw instructions to picture for card with id {}", card.id)))?;
        Ok(SkiaCard {
            drawing_commands: finished_picture,
            project,
            geometry: &layout.geometry,
        })
    }

    fn write_png<'a, P>(&self, card: &Self::SingleCard<'a>, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<std::path::Path> + Debug {
        // Prepare the raster surface
        let path_str = format!("{:?}", path);
        let mut surface =
            Surface::new_raster_n32_premul((card.geometry.width as i32, card.geometry.height as i32))
            .ok_or_else(|| SkiaRendererError::GraphicsError(format!("could not create a surface for writing ({path_str})")))?;

        // Draw the saved picture
        card.drawing_commands.playback(surface.canvas());

        // Create an image from the surface
        let image = surface.image_snapshot();

        // Encode the image as a png
        let image_data =
            image
                .encode(surface.direct_context(), EncodedImageFormat::PNG, Some(100))
                .ok_or_else(|| SkiaRendererError::GraphicsError(format!("could not encode card image as png ({path_str})")))?;
        
        // Write out the file
        let mut png_file = File::create(path).into_diagnostic()?;
        png_file.write_all(image_data.as_bytes()).into_diagnostic()?;
        png_file.flush().into_diagnostic()?;

        Ok(())
    }

    fn write_single_pdf<'a, P>(&self, card: &Self::SingleCard<'a>, path: P) -> Result<(), Self::Error>
    where
        P: AsRef<std::path::Path> + Debug {
        let document_width = units::pixels_to_points(card.geometry.width, card.geometry.dpi);
        let document_height = units::pixels_to_points(card.geometry.height, card.geometry.dpi);
        let card_scale = units::scale_factor_at_dpi(card.geometry.dpi);

        let document = skia_safe::pdf::new_document(Some(&pdf::metadata_from_project(card.project)));
        let mut document = document.begin_page((document_width, document_height), None);
        document.canvas().scale((card_scale, card_scale));
        card.drawing_commands.playback(document.canvas());
        let document = document.end_page();
        let pdf_data = document.close();

        let mut pdf_file = File::create(path).into_diagnostic()?;
        pdf_file.write_all(pdf_data.as_bytes()).into_diagnostic()?;
        pdf_file.flush().into_diagnostic()?;

        Ok(())
    }

    fn write_deck_pdf<'a, I, P>(&self, cards: I, path: P, sheet: &Sheet) -> Result<(), Self::Error>
    where
        I: Iterator<Item = Self::SingleCard<'a>>,
        P: AsRef<std::path::Path> + Debug {
        let mut cards = cards.peekable();
        let project =
            if let Some(first_card) = cards.peek() {
                first_card.project
            } else {
                // rendering no cards is probably fine, but we shouldn't output
                // a file in that case.
                return Ok(())
            };
        let document = skia_safe::pdf::new_document(Some(&pdf::metadata_from_project(project)));
        let document = pdf::draw_cards_in_document(cards, document, sheet)?;
        let pdf_data = document.close();

        let mut pdf_file = File::create(path).into_diagnostic()?;
        pdf_file.write_all(pdf_data.as_bytes()).into_diagnostic()?;
        pdf_file.flush().into_diagnostic()?;

        Ok(())
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum SkiaRendererError {
    #[error("encountered an error while rendering a card: {0}")]
    GraphicsError(String),
}

pub struct SkiaCard<'a> {
    drawing_commands: Picture,
    project: &'a Project,
    geometry: &'a Geometry,
}
