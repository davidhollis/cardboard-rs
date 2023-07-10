use skia_safe::{Canvas, Paint, Color4f, IRect, PaintStyle, textlayout::{TextStyle, FontCollection, ParagraphBuilder, ParagraphStyle}, FontMgr, Rect, ClipOp, Color as SkiaColor, PathEffect};

use crate::{layout::{Element, Rectangle, Text, Box, model::styles::{color::{ColorRef, Color as CardboardColor}, stroke::DashPattern}, PathStyle, Stroke, Solid}, data::Card};

use super::SkiaRendererError;

pub fn draw_elements(card: &Card, elements: &Vec<Element>, canvas: &mut Canvas, frame_width: usize, frame_height: usize) -> Result<(), miette::Error> {
    for element in elements {
        match element {
            Element::Background(bg) => draw_rect(&bg.to_rect(frame_width, frame_height), card, canvas)?,
            Element::Rectangle(rect) => draw_rect(rect, card, canvas)?,
            Element::Text(text) => draw_text(text, card, canvas)?,
            Element::Box(bx) => draw_box(card, bx, canvas)?,
        }
    }
    Ok(())
}

fn draw_rect(rect: &Rectangle, card: &Card, canvas: &mut Canvas) -> Result<(), miette::Error> {
    let (fill, stroke) = compute_path_styles(&rect.style, card)?;
    let irect = IRect::from_xywh(rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32);

    if let Some(fill) = fill {
        canvas.draw_irect(irect, &fill);
    }

    if let Some(stroke) = stroke {
        canvas.draw_irect(irect, &stroke);
    }

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
    fixed_text_style.set_font_size(42.0f32); // 42px ~ 10pt ~ 10/72in @ 300 dpi
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

fn resolve_color_ref(card: &Card, color_ref: &ColorRef) -> Result<SkiaColor, miette::Error> {
    let color = match color_ref {
        ColorRef::Named(name_template) => {
            let name = name_template.render(card.try_into()?)?;
            card.color_named(name.as_str()).ok_or_else(|| SkiaRendererError::InvalidColor(name))
        },
        ColorRef::Static(cb_color) => Ok(cb_color.clone()),
    }.map(|cb_color| match cb_color {
        CardboardColor::RGB(r, g, b) => SkiaColor::from_rgb(r, g, b),
        CardboardColor::RGBA(r, g, b, a) => SkiaColor::from_argb(a, r, g, b),
    });

    Ok(color?)
}

fn compute_path_styles(styles: &Vec<PathStyle>, card: &Card) -> Result<(Option<Paint>, Option<Paint>), miette::Error> {
    let mut fill_paint = Paint::new(Into::<Color4f>::into(SkiaColor::TRANSPARENT), None);
    let mut should_fill = false;
    let mut stroke_paint = Paint::new(Into::<Color4f>::into(SkiaColor::TRANSPARENT), None);
    let mut should_stroke = false;
    let mut should_render_at_all = true;
    let card_ctx = TryInto::<&handlebars::Context>::try_into(card)?;

    for style in styles {
        match style {
            PathStyle::Stroke(Stroke { width, color, pattern }) => {
                should_stroke = true;
                stroke_paint.set_style(PaintStyle::Stroke);
                stroke_paint.set_stroke(true);
                stroke_paint.set_anti_alias(true);
                stroke_paint.set_color(resolve_color_ref(card, color)?);
                stroke_paint.set_stroke_width(*width as f32);
                match pattern {
                    DashPattern::Solid => {},
                    DashPattern::Dashed(segments) => {
                        let path_effect =
                            PathEffect::dash(
                                segments.iter().map(|len| (*len as f32) * (*width as f32)).collect::<Vec<f32>>().as_slice(),
                                0.0f32
                            )
                            .ok_or_else(|| SkiaRendererError::GraphicsError("could not build dash pattern".to_string()))?;
                        stroke_paint.set_path_effect(path_effect);
                    }
                }
            },
            PathStyle::Solid(Solid { color }) => {
                should_fill = true;
                fill_paint.set_style(PaintStyle::Fill);
                fill_paint.set_color(resolve_color_ref(card, color)?);
            },
            PathStyle::OnlyIf(condition) => {
                should_render_at_all = should_render_at_all && condition.evaluate(card_ctx)?;
            }
        }
    }

    if should_render_at_all {
        Ok((
            if should_fill { Some(fill_paint) } else { None },
            if should_stroke { Some(stroke_paint) } else { None },
        ))
    } else {
        Ok((None, None))
    }
}
