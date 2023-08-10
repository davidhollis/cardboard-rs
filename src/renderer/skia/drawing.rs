use skia_safe::{Canvas, Paint, Color4f, IRect, PaintStyle, textlayout::{TextStyle as SkTextStyle, FontCollection, ParagraphBuilder, ParagraphStyle}, FontMgr, Rect, ClipOp, Color as SkiaColor, PathEffect, FontStyle, font_style::Slant};

use crate::{layout::{Element, Rectangle, Text, Box, model::styles::{color::{ColorRef, Color as CardboardColor}, stroke::DashPattern, text::{Foreground, Background as TextBackground, Align, Alignment}, font::{Weight, Width}}, PathStyle, Stroke, Solid, TextStyle, Font}, data::Card};

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
    // TODO: eventually support embedded markup to control styles
    // TODO: eventually support embedded icons

    if let Some(paragraph_style) = compute_text_styles(&text.style, card)? {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);

        // Resolve the template and add the text to the builder
        let filled_template = text.contents.render(card.try_into()?)?;
        paragraph_builder.add_text(filled_template.as_str());

        // Lay out and draw the paragraph
        let mut paragraph = paragraph_builder.build();
        paragraph.layout(text.frame.w as f32);
        paragraph.paint(canvas, (text.frame.x as f32, text.frame.y as f32));
    }

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

fn compute_text_styles(styles: &Vec<TextStyle>, card: &Card) -> Result<Option<ParagraphStyle>, miette::Error> {
    let mut text_style = SkTextStyle::new();
    let mut text_align = skia_safe::textlayout::TextAlign::Left;
    let mut should_render = true;
    let card_ctx = TryInto::<&handlebars::Context>::try_into(card)?;

    // Default styles
    let mut default_foreground_paint = Paint::new(
        Into::<Color4f>::into(SkiaColor::BLACK),
        None
    );
    default_foreground_paint.set_anti_alias(true);
    text_style.set_foreground_color(&default_foreground_paint);

    // User-defined styles
    for style in styles {
        match style {
            TextStyle::Font(Font {family, weight, width, style}) => {
                text_style.set_font_families(&[family]);
                text_style.set_font_style(FontStyle::new(
                    match weight {
                        Weight::Thin => skia_safe::font_style::Weight::THIN,
                        Weight::ExtraLight => skia_safe::font_style::Weight::EXTRA_LIGHT,
                        Weight::Light => skia_safe::font_style::Weight::LIGHT,
                        Weight::Normal => skia_safe::font_style::Weight::NORMAL,
                        Weight::Medium => skia_safe::font_style::Weight::MEDIUM,
                        Weight::SemiBold => skia_safe::font_style::Weight::SEMI_BOLD,
                        Weight::Bold => skia_safe::font_style::Weight::BOLD,
                        Weight::ExtraBold => skia_safe::font_style::Weight::EXTRA_BOLD,
                        Weight::Black => skia_safe::font_style::Weight::BLACK,
                        Weight::ExtraBlack => skia_safe::font_style::Weight::EXTRA_BLACK,
                    },
                    match width {
                        Width::UltraCondensed => skia_safe::font_style::Width::ULTRA_CONDENSED,
                        Width::Condensed => skia_safe::font_style::Width::CONDENSED,
                        Width::SemiCondensed => skia_safe::font_style::Width::SEMI_CONDENSED,
                        Width::Normal => skia_safe::font_style::Width::NORMAL,
                        Width::SemiWide => skia_safe::font_style::Width::SEMI_EXPANDED,
                        Width::Wide => skia_safe::font_style::Width::EXPANDED,
                        Width::UltraWide => skia_safe::font_style::Width::ULTRA_EXPANDED,
                    },
                    if style == "italic" { Slant::Italic } else { Slant::Upright }
                ));
            },
            TextStyle::Size(sz) => {
                text_style.set_font_size(sz.pixel_size());
            },
            TextStyle::Align(Align {alignment}) => {
                text_align = match alignment {
                    Alignment::Left => skia_safe::textlayout::TextAlign::Left,
                    Alignment::Center => skia_safe::textlayout::TextAlign::Center,
                    Alignment::Right => skia_safe::textlayout::TextAlign::Right,
                    Alignment::Justify => skia_safe::textlayout::TextAlign::Justify,
                }
            },
            TextStyle::Foreground(Foreground {color}) => {
                let mut foreground_paint = Paint::new(
                    Into::<Color4f>::into(resolve_color_ref(card, color)?),
                    None
                );
                foreground_paint.set_anti_alias(true);
                text_style.set_foreground_color(&foreground_paint);
            },
            TextStyle::Background(TextBackground {color}) => {
                let mut background_paint = Paint::new(
                    Into::<Color4f>::into(resolve_color_ref(card, color)?),
                    None
                );
                background_paint.set_anti_alias(true);
                text_style.set_background_color(&background_paint);
            },
            TextStyle::OnlyIf(condition) => {
                should_render = should_render && condition.evaluate(card_ctx)?;
            }
        }
    }

    if should_render {
        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        paragraph_style.set_text_align(text_align);
        Ok(Some(paragraph_style))
    } else {
        Ok(None)
    }
}
