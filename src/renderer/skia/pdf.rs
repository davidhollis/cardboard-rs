use chrono::{Datelike, Timelike};
use lazy_static::lazy_static;
use skia_safe::{pdf::Metadata, DateTime, Document, document::state::Open, Canvas, Rect, Paint, Color4f, Color};

use crate::{data::project::Project, config::sheets::{layout::{Sheet, CropLineOrientation}, units}};

use super::{SkiaCard, SkiaRendererError};

lazy_static! {
    static ref PDF_CREATOR_FIELD: String = format!("cardboard v{}", env!("CARGO_PKG_VERSION"));
}

pub(super) fn metadata_from_project(project: &Project) -> Metadata {
    let mut metadata = Metadata::default();

    if let Some(ref author) = project.pdf_metadata.author {
        metadata.author = author.clone();
    }
    if let Some(ref title) = project.pdf_metadata.title {
        metadata.title = title.clone();
    }
    if let Some(ref subject) = project.pdf_metadata.subject {
        metadata.subject = subject.clone();
    }
    if let Some(ref keywords) = project.pdf_metadata.keywords {
        metadata.keywords = keywords.clone();
    }

    metadata.creator = PDF_CREATOR_FIELD.clone();
    metadata.producer = PDF_CREATOR_FIELD.clone();
    let now = get_current_time_as_skia();
    metadata.creation = Some(now.clone());
    metadata.modified = Some(now);

    metadata
}

fn get_current_time_as_skia() -> DateTime {
    let now_local = chrono::offset::Local::now();
    DateTime {
        time_zone_minutes: (now_local.offset().local_minus_utc()/60) as i16,
        year: now_local.year() as u16,
        month: now_local.month() as u8,
        day_of_week: now_local.weekday().number_from_monday() as u8,
        day: now_local.day() as u8,
        hour: now_local.hour() as u8,
        minute: now_local.minute() as u8,
        second: now_local.second() as u8,
    }
}

pub(super) fn draw_cards_in_document<'a, I>(cards: I, document: Document<Open>, sheet: &Sheet) -> miette::Result<Document<Open>>
where I: Iterator<Item = SkiaCard<'a>> {
    let cards_per_sheet = sheet.num_cards();
    let page_size = (sheet.page_size.width, sheet.page_size.height);
    let mut document = document.begin_page(page_size, None);

    draw_crop_lines(document.canvas(), sheet);

    for (idx_in_set, card) in cards.enumerate() {
        let idx_on_sheet = idx_in_set % cards_per_sheet;
        let sheet_number = idx_in_set / cards_per_sheet;

        if idx_on_sheet == 0 && sheet_number > 0 {
            // When we encounter the first card on every sheet that isn't the
            // first one, open a new page
            document = document.end_page().begin_page(page_size, None);
            draw_crop_lines(document.canvas(), sheet);
        }

        let slot =
            sheet.cards
                .get(idx_on_sheet)
                .ok_or(SkiaRendererError::GraphicsError(
                    format!("something went wrong with the sheet math: tried to access card index {} on a sheet with {} cards", idx_on_sheet, cards_per_sheet)
                ))?;

        // Save the canvas state
        let canvas = document.canvas();
        canvas.save();

        // TODO(#25): support the `rotate` and `reflect` properties of config::sheets::layout::CardPlacement
        // https://github.com/davidhollis/cardboard-rs/issues/25

        // Translate the canvas so the origin is now the upper left corner of the card
        canvas.translate((slot.x, slot.y));

        // Scale the canvas so 1 unit = 1 pixel
        let card_scale = units::scale_factor_at_dpi(card.geometry.dpi);
        canvas.scale((card_scale, card_scale));

        // Translate back so the origin is at the intersection of the top and left cut lines
        canvas.translate((-(card.geometry.cut.left as f32), -(card.geometry.cut.top as f32)));

        // Set a clipping mask that covers the area inside the cut lines
        // TODO(#26): Make this configurable. Sometimes we might want to draw the overflow in the gutter
        // https://github.com/davidhollis/cardboard-rs/issues/26
        let (content_width, content_height) = card.geometry.content_size();
        canvas.clip_rect(
            Rect::from_xywh(0., 0., content_width as f32, content_height as f32),
            Some(skia_safe::ClipOp::Intersect),
            Some(true),
        );

        // Draw the card
        card.drawing_commands.playback(canvas);

        // Reset the canvas state
        canvas.restore();
    }

    Ok(document.end_page())
}

fn draw_crop_lines(canvas: &mut Canvas, sheet: &Sheet) -> () {
    let mut crop_line_paint = Paint::new(Color4f::from(Color::BLACK), None);
    crop_line_paint
        .set_style(skia_safe::PaintStyle::Stroke)
        .set_anti_alias(true)
        .set_stroke(true)
        .set_stroke_width(0.5);
    
    for crop_line in &sheet.crop_lines {
        match crop_line.orientation {
            CropLineOrientation::Horizontal => {
                if crop_line.length > (sheet.page_size.width / 2.) {
                    // Draw a crop line all the way across the page
                    canvas.draw_line(
                        (0., crop_line.offset),
                        (sheet.page_size.width, crop_line.offset),
                        &crop_line_paint,
                    );
                } else {
                    // Draw two smaller crop lines, one at each edge
                    canvas.draw_line(
                        (0., crop_line.offset),
                        (crop_line.length, crop_line.offset),
                        &crop_line_paint,
                    );
                    canvas.draw_line(
                        (sheet.page_size.width - crop_line.length, crop_line.offset),
                        (sheet.page_size.width, crop_line.offset),
                        &crop_line_paint,
                    );
                }
            },
            CropLineOrientation::Vertical => {
                if crop_line.length > (sheet.page_size.height / 2.) {
                    // Draw a crop line all the way up and down the page
                    canvas.draw_line(
                        (crop_line.offset, 0.),
                        (crop_line.offset, sheet.page_size.height),
                        &crop_line_paint,
                    );
                } else {
                    // Draw two smaller crop lines, one at each edge
                    canvas.draw_line(
                        (crop_line.offset, 0.),
                        (crop_line.offset, crop_line.length),
                        &crop_line_paint,
                    );
                    canvas.draw_line(
                        (crop_line.offset, sheet.page_size.height - crop_line.length),
                        (crop_line.offset, sheet.page_size.height),
                        &crop_line_paint,
                    );
                }
            },
        }
    }
}
