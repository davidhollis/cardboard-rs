use std::{fmt::Debug, fs::File, io::Write, collections::HashMap, sync::Arc};

use miette::{Diagnostic, IntoDiagnostic};
use skia_safe::{EncodedImageFormat, PictureRecorder, Rect, Picture, Surface, Image, images, Data};
use thiserror::Error;

use crate::{data::{project::Project}, config::sheets::{units, layout::Sheet}, layout::model::{geometry::Geometry, styles::text::FlatTextStyle}};

use super::Renderer;

mod drawing;
mod pdf;

pub struct SkiaRenderer {
    images: HashMap<String, Arc<Image>>,
}

impl SkiaRenderer {
    pub fn new() -> SkiaRenderer {
        SkiaRenderer { images: HashMap::new() }
    }

    pub fn load_image(&mut self, image_name: &str, project: &Project) -> miette::Result<Arc<Image>> {
        match self.images.get(image_name) {
            Some(img) => Ok(img.clone()),
            None => {
                log::debug!("Encountered image {} for the first time", image_name);
                let img_path = project
                    .full_image_path(image_name)
                    .ok_or(SkiaRendererError::NoSuchImage(image_name.to_string()))?;
                let image_data = std::fs::read(img_path).into_diagnostic()?;
                let loaded_image =
                    images::deferred_from_encoded_data(Data::new_copy(image_data.as_slice()), None)
                    .ok_or(SkiaRendererError::GraphicsError(format!("failed to decode image from contents of file {}", img_path)))?;
                log::debug!("Loaded image with size {}x{}", loaded_image.width(), loaded_image.height());
                let rc_image = Arc::new(loaded_image);
                self.images.insert(image_name.to_string(), rc_image.clone());
                Ok(rc_image)
            }
        }
    }
}

impl Renderer for SkiaRenderer {
    type SingleCard<'a> = SkiaCard<'a>;
    type Error = miette::Report;

    fn render_single<'a>(&mut self, project: &'a Project, card_id: &str) -> Result<Self::SingleCard<'a>, Self::Error> {
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
        let mut base_text_styles = FlatTextStyle::default();
        if let Some(ref base_style_definitions) = layout.base_text {
            base_text_styles.apply(base_style_definitions.styles.as_slice());
        }
        let mut render_ctx = drawing::CardRenderContext::new(card, project, layout.geometry.dpi, self, base_text_styles);
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
        Self::SingleCard<'a>: 'a,
        I: Iterator<Item = &'a Self::SingleCard<'a>>,
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
    #[error("no image named {0} was found in the project directory")]
    NoSuchImage(String),
}

pub struct SkiaCard<'a> {
    drawing_commands: Picture,
    project: &'a Project,
    geometry: &'a Geometry,
}
