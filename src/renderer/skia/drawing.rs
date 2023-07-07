use skia_safe::{Canvas, Paint, Color4f, IRect, PaintStyle, textlayout::{TextStyle, FontCollection, ParagraphBuilder, ParagraphStyle}, FontMgr, Rect, ClipOp};

use crate::{layout::{Element, Rectangle, Text, Box}, data::Card};

pub fn draw_elements(card: &Card, elements: &Vec<Element>, canvas: &mut Canvas, frame_width: usize, frame_height: usize) -> Result<(), miette::Error> {
    for element in elements {
        match element {
            Element::Background(bg) => draw_rect(&bg.to_rect(frame_width, frame_height), canvas)?,
            Element::Rectangle(rect) => draw_rect(rect, canvas)?,
            Element::Text(text) => draw_text(text, card, canvas)?,
            Element::Box(bx) => draw_box(card, bx, canvas)?,
        }
    }
    Ok(())
}

fn draw_rect(rect: &Rectangle, canvas: &mut Canvas) -> Result<(), miette::Error> {
    // TODO: this just strokes the rectangle in black.
    //       We should aggregrate the styles, then do a 2-pass fill then stroke
    let mut paint = Paint::new(Color4f::new(0.0f32,0.0f32,0.0f32,1.0f32), None);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(1.0f32);
    let irect = IRect::from_xywh(rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32);
    canvas.draw_irect(irect, &paint);
    Ok(())
}

fn draw_text(text: &Text, card: &Card, canvas: &mut Canvas) -> Result<(), miette::Error> {
    // TODO: this just draw the text in black at size 10.
    //       We should aggregate styles and actually manage fonts
    // TODO: eventually support embedded markup to control styles
    // TODO: eventually support embedded icons

    // Set up default styles
    let mut fixed_text_style = TextStyle::new();
    let mut paint = Paint::new(Color4f::new(0.0f32, 0.0f32, 0.0f32, 1.0f32), None);
    paint.set_anti_alias(true);
    fixed_text_style.set_foreground_color(&paint);
    fixed_text_style.set_font_size(10.0f32);
    let mut paragraph_style = ParagraphStyle::new();
    paragraph_style.set_text_style(&fixed_text_style);
    let mut default_font_collection = FontCollection::new();
    default_font_collection.set_default_font_manager(FontMgr::new(), None);
    let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, default_font_collection);

    // Resolve the template and add the text to the builder
    let filled_template = text.contents.render(card.try_into()?)?;
    paragraph_builder.add_text(filled_template.as_str());

    // Lay out and draw the paragraph
    let mut paragraph = paragraph_builder.build();
    paragraph.layout(text.frame.w as f32);
    paragraph.paint(canvas, (text.frame.x as f32, text.frame.y as f32));

    Ok(())
}

fn draw_box(card: &Card, bx: &Box, canvas: &mut Canvas) -> Result<(), miette::Error> {
    canvas.save();
    canvas.translate((bx.x as f32, bx.y as f32));
    canvas.clip_rect(Rect::from_iwh(bx.w as i32, bx.h as i32), ClipOp::Intersect, Some(true));

    draw_elements(card, &bx.contents, canvas, bx.w, bx.h)?;

    canvas.restore();

    Ok(())
}