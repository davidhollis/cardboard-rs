use skia_safe::{Canvas, Paint, Color4f, IRect, PaintStyle, textlayout::{TextStyle as SkTextStyle, FontCollection, ParagraphBuilder, ParagraphStyle}, FontMgr, Rect, ClipOp, Color as SkiaColor, PathEffect, FontStyle, font_style::Slant};

use crate::{layout::model::{elements::{Element, shapes::Rectangle, text::Text, containers::Box, image::{Image, Scale}, Frame}, styles::{color::{ColorRef, Color as CardboardColor}, stroke::DashPattern, text::{Foreground, Background as TextBackground, Alignment, ComputedTextStyle, Size, Units}, font::{Weight, Width}, PathStyle, stroke::Stroke, solid::Solid}}, data::{card::Card, project::{Project}}, format::{self, FormattedTextInstruction}};

use super::{SkiaRendererError, SkiaRenderer};

pub struct CardRenderContext<'a> {
    card: &'a Card,
    project: &'a Project,
    dpi: usize,
    renderer: &'a mut SkiaRenderer,
    base_text_styles: ComputedTextStyle<'a>,
}

impl<'a> CardRenderContext<'a> {
    pub fn new(card: &'a Card, project: &'a Project, dpi: usize, renderer: &'a mut SkiaRenderer, base_text_styles: ComputedTextStyle<'a>) -> CardRenderContext<'a> {
        CardRenderContext { card, project, dpi, renderer, base_text_styles }
    }

    pub fn draw_elements(&mut self, canvas: &mut Canvas, elements: &Vec<Element>, frame_width: usize, frame_height: usize) -> Result<(), miette::Error> {
        for element in elements {
            match element {
                Element::Background(bg) => self.draw_rect(canvas, &bg.to_rect(frame_width, frame_height))?,
                Element::Rectangle(rect) => self.draw_rect(canvas, rect)?,
                Element::Image(image_frame) => self.draw_image(canvas, image_frame)?,
                Element::Text(text) => self.draw_text(canvas, text)?,
                Element::Box(bx) => self.draw_box(canvas, bx)?,
            }
        }
        Ok(())
    }
    
    fn draw_rect(&self, canvas: &mut Canvas, rect: &Rectangle) -> Result<(), miette::Error> {
        let (fill, stroke) = self.compute_path_styles(&rect.style)?;
        let irect = IRect::from_xywh(rect.x as i32, rect.y as i32, rect.w as i32, rect.h as i32);
    
        if let Some(fill) = fill {
            canvas.draw_irect(irect, &fill);
        }
    
        if let Some(stroke) = stroke {
            canvas.draw_irect(irect, &stroke);
        }
    
        Ok(())
    }

    fn draw_image(&mut self, canvas: &mut Canvas, image_frame: &Image) -> Result<(), miette::Error> {
        let image_name = image_frame.name.render(self.card.try_into()?)?;
        let image = self.renderer.load_image(&image_name, self.project)?;

        let image = match image {
            Some(image) => image,
            None => {
                self.draw_image_placeholder(canvas, &image_frame.frame, &image_name);
                return Ok(())
            }
        };

        let horizontal_scale_factor = (image_frame.frame.w as f32)/(image.width() as f32);
        let vertical_scale_factor = (image_frame.frame.h as f32)/(image.height() as f32);
        let (frame_center_x, frame_center_y) = image_frame.frame.center();

        let mut paint = Paint::new(Color4f::from(SkiaColor::BLACK), None);
        paint.set_anti_alias(true);

        match image_frame.scale {
            Scale::Fit => {
                let actual_scale_factor = horizontal_scale_factor.min(vertical_scale_factor);
                let scaled_width = (image.width() as f32) * actual_scale_factor;
                let scaled_height = (image.height() as f32) * actual_scale_factor;
                let scaled_image_bounds = Rect::from_xywh(
                    (frame_center_x as f32) - (scaled_width / 2.),
                    (frame_center_y as f32) - (scaled_height / 2.),
                    scaled_width,
                    scaled_height,
                );

                canvas.draw_image_rect(image, None, &scaled_image_bounds, &paint);
            },
            Scale::Fill => {
                let actual_scale_factor = horizontal_scale_factor.max(vertical_scale_factor);
                let scaled_width = (image.width() as f32) * actual_scale_factor;
                let scaled_height = (image.height() as f32) * actual_scale_factor;
                let scaled_image_bounds = Rect::from_xywh(
                    (frame_center_x as f32) - (scaled_width / 2.),
                    (frame_center_y as f32) - (scaled_height / 2.),
                    scaled_width,
                    scaled_height,
                );

                canvas.save();
                canvas.clip_irect(
                    IRect::from_xywh(
                        image_frame.frame.x as i32,
                        image_frame.frame.y as i32,
                        image_frame.frame.w as i32,
                        image_frame.frame.h as i32,
                    ),
                    ClipOp::Intersect,
                );
                canvas.draw_image_rect(image, None, &scaled_image_bounds, &paint);
                canvas.restore();
            },
            Scale::Stretch => {
                canvas.draw_image_rect(image, None, Rect::from_xywh(
                    image_frame.frame.x as f32,
                    image_frame.frame.y as f32,
                    image_frame.frame.w as f32,
                    image_frame.frame.h as f32,
                ), &paint);
            },
            Scale::None => {
                let unscaled_image_bounds = Rect::from_xywh(
                    (frame_center_x as f32) - ((image.width() as f32) / 2.),
                    (frame_center_y as f32) - ((image.height() as f32) / 2.),
                    image.width() as f32,
                    image.height() as f32,
                );

                canvas.save();
                canvas.clip_irect(
                    IRect::from_xywh(
                        image_frame.frame.x as i32,
                        image_frame.frame.y as i32,
                        image_frame.frame.w as i32,
                        image_frame.frame.h as i32,
                    ),
                    ClipOp::Intersect,
                );
                canvas.draw_image_rect(image, None, &unscaled_image_bounds, &paint);
                canvas.restore();
            },
        }
        Ok(())
    }
    
    fn draw_image_placeholder(&self, canvas: &mut Canvas, frame: &Frame, image_name: &str) -> () {
        let mut foreground_paint = Paint::new(Color4f::from(SkiaColor::BLUE), None);
        foreground_paint.set_style(PaintStyle::Stroke);
        foreground_paint.set_stroke(true);
        foreground_paint.set_anti_alias(true);
        foreground_paint.set_stroke_width(1.);
        let mut background_paint = Paint::new(Color4f::from(SkiaColor::WHITE), None);
        background_paint.set_style(PaintStyle::Fill);
        background_paint.set_stroke(false);
        background_paint.set_anti_alias(true);
        let frame_irect = IRect::from_xywh(
            frame.x as i32,
            frame.y as i32,
            frame.w as i32,
            frame.h as i32,
        );

        canvas.draw_irect(frame_irect, &background_paint);
        canvas.draw_irect(frame_irect, &foreground_paint);
        canvas.draw_line(
            (frame.x as i32, frame.y as i32),
            ((frame.x + frame.w) as i32, (frame.y + frame.h) as i32),
            &foreground_paint,
        );
        canvas.draw_line(
            ((frame.x + frame.w) as i32, frame.y as i32),
            (frame.x as i32, (frame.y + frame.h) as i32),
            &foreground_paint,
        );

        foreground_paint.set_style(PaintStyle::Fill);
        foreground_paint.set_stroke(false);
        let mut text_style = SkTextStyle::new();
        text_style.set_foreground_color(&foreground_paint);
        text_style.set_background_color(&background_paint);
        let text_size = Size { size: 6, units: Units::Points }.pixel_size(self.dpi);
        text_style.set_font_size(text_size);
        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        paragraph_style.set_text_align(skia_safe::textlayout::TextAlign::Left);
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
        paragraph_builder.add_text(image_name);
        let mut paragraph = paragraph_builder.build();
        paragraph.layout(frame.w as f32);

        canvas.save();
        canvas.clip_irect(frame_irect, ClipOp::Intersect);
        paragraph.paint(canvas, (frame.x as f32, frame.y as f32));
        canvas.restore();
    }
    
    fn draw_text(&self, canvas: &mut Canvas, text: &Text) -> Result<(), miette::Error> {
        // TODO(#13): eventually support embedded markup to control styles
        // https://github.com/davidhollis/cardboard-rs/issues/13
        // TODO(#28): eventually support embedded icons
        // https://github.com/davidhollis/cardboard-rs/issues/28

        let mut style_stack: Vec<(&str, ComputedTextStyle<'_>)> = vec![];

        // Build the text styles:
        // 1. Start with the layout's base styles
        let mut text_styles = self.base_text_styles.clone();

        // 2. If the Text element has a named style that corresponds to one
        //    that's in the project's style registry, apply those styles
        if let Some(named_style) = text.style.as_ref().and_then(|style_name| self.project.style_set_for(style_name)) {
            text_styles.apply(named_style);
        }

        // 3. Then apply the inline styles
        text_styles.apply(text.inline_styles.as_slice());
        style_stack.push(("", text_styles.clone()));
    
        if let Some(paragraph_style) = self.skia_text_styles(text_styles)? {
            let mut font_collection = FontCollection::new();
            font_collection.set_default_font_manager(FontMgr::new(), None);
            let mut paragraph_builder = ParagraphBuilder::new(&paragraph_style, font_collection);
    
            // Resolve the template and add the text to the builder
            let filled_template = text.contents.render(self.card.try_into()?)?;
            let formatted = format::parse(&filled_template);
            for instruction in &formatted {
                match instruction {
                    FormattedTextInstruction::AddText(ref text) => {
                        paragraph_builder.add_text(text);
                    },
                    FormattedTextInstruction::PushStyle(ref style_name) => {
                        if let Some(style_definition) = self.project.style_set_for(&style_name) {
                            if let Some((_, ref previous_text_style)) = style_stack.last() {
                                let mut new_text_style = previous_text_style.clone();
                                new_text_style.apply(style_definition);
                                if let Some(new_paragraph_style) = self.skia_text_styles(new_text_style.clone())? {
                                    paragraph_builder.push_style(new_paragraph_style.text_style());
                                    style_stack.push((style_name, new_text_style));
                                } else {
                                    log::warn!("While rendering card {}: style <{}> failed one or more only-if rules, ignoring tag.", self.card.id, style_name);
                                }
                            } else {
                                log::warn!("While rendering card {}: no previous state found whn trying to apply style <{}>. Did you close too many tags?", self.card.id, style_name);
                            }
                        } else {
                            log::warn!("While rendering card {}: no style definition found for tag <{}>", self.card.id, style_name);
                        }
                    },
                    FormattedTextInstruction::PopStyle(ref style_name) => {
                        if let Some((most_recent_style_name, _)) = style_stack.pop() {
                            paragraph_builder.pop();
                            if most_recent_style_name != style_name {
                                log::warn!("While rendering card {}: encountered </{}>, but the most recent tag was <{}>. Formatting may be incorrect", self.card.id, style_name, most_recent_style_name);
                            }
                        } else {
                            log::warn!("While rendering card {}: encountered </{}>, but there was no open tag to close.", self.card.id, style_name);
                        }
                    },
                    FormattedTextInstruction::InsertPlaceholder(ref symbol_name) => {
                        log::warn!("While rendering card {}: encountered :{}:, but symbols are not supported yet. Ignoring.", self.card.id, symbol_name);
                        paragraph_builder.add_text(format!(":{}:", symbol_name));
                    }
                }
            }
    
            // Lay out and draw the paragraph
            let mut paragraph = paragraph_builder.build();
            paragraph.layout(text.frame.w as f32);
            paragraph.paint(canvas, (text.frame.x as f32, text.frame.y as f32));
        }
    
        Ok(())
    }
    
    fn draw_box(&mut self, canvas: &mut Canvas, bx: &Box) -> Result<(), miette::Error> {
        canvas.save();
        canvas.translate((bx.x as f32, bx.y as f32));
        canvas.clip_rect(Rect::from_iwh(bx.w as i32, bx.h as i32), ClipOp::Intersect, Some(true));
    
        self.draw_elements(canvas, &bx.contents, bx.w, bx.h)?;
    
        canvas.restore();
    
        Ok(())
    }
    
    fn resolve_color_ref(&self, color_ref: &ColorRef) -> Result<SkiaColor, miette::Error> {
        let color = match color_ref {
            ColorRef::Named(name_template) => {
                let name = name_template.render(self.card.try_into()?)?;
                self.project.color_named(name.as_str())
            },
            ColorRef::Static(cb_color) => Ok(cb_color.clone()),
        }.map(|cb_color| match cb_color {
            CardboardColor::RGB(r, g, b) => SkiaColor::from_rgb(r, g, b),
            CardboardColor::RGBA(r, g, b, a) => SkiaColor::from_argb(a, r, g, b),
        });
    
        Ok(color?)
    }
    
    fn compute_path_styles(&self, styles: &Vec<PathStyle>) -> Result<(Option<Paint>, Option<Paint>), miette::Error> {
        let mut fill_paint = Paint::new(Into::<Color4f>::into(SkiaColor::TRANSPARENT), None);
        let mut should_fill = false;
        let mut stroke_paint = Paint::new(Into::<Color4f>::into(SkiaColor::TRANSPARENT), None);
        let mut should_stroke = false;
        let mut should_render_at_all = true;
        let card_ctx = TryInto::<&handlebars::Context>::try_into(self.card)?;
    
        for style in styles {
            match style {
                PathStyle::Stroke(Stroke { width, color, pattern }) => {
                    should_stroke = true;
                    stroke_paint.set_style(PaintStyle::Stroke);
                    stroke_paint.set_stroke(true);
                    stroke_paint.set_anti_alias(true);
                    stroke_paint.set_color(self.resolve_color_ref(color)?);
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
                    fill_paint.set_color(self.resolve_color_ref(color)?);
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
    
    fn skia_text_styles(&self, styles: ComputedTextStyle<'_>) -> Result<Option<ParagraphStyle>, miette::Error> {
        let mut should_render = true;
        let card_ctx = TryInto::<&handlebars::Context>::try_into(self.card)?;
        
        for cond in styles.conditions {
            should_render = should_render && cond.evaluate(card_ctx)?;
        }
        
        if !should_render {
            return Ok(None)
        }
        
        // Default styles
        let mut text_style = SkTextStyle::new();
        let mut default_foreground_paint = Paint::new(
            Into::<Color4f>::into(SkiaColor::BLACK),
            None
        );
        default_foreground_paint.set_anti_alias(true);
        text_style.set_foreground_color(&default_foreground_paint);
    
        // User-defined styles
        if let Some(family) = styles.font_family {
            text_style.set_font_families(&[family]);
        }
        text_style.set_font_style(FontStyle::new(
            match styles.font_weight {
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
            match styles.font_width {
                Width::UltraCondensed => skia_safe::font_style::Width::ULTRA_CONDENSED,
                Width::Condensed => skia_safe::font_style::Width::CONDENSED,
                Width::SemiCondensed => skia_safe::font_style::Width::SEMI_CONDENSED,
                Width::Normal => skia_safe::font_style::Width::NORMAL,
                Width::SemiWide => skia_safe::font_style::Width::SEMI_EXPANDED,
                Width::Wide => skia_safe::font_style::Width::EXPANDED,
                Width::UltraWide => skia_safe::font_style::Width::ULTRA_EXPANDED,
            },
            if styles.font_style == Some("italic") { Slant::Italic } else { Slant::Upright },
        ));
        if let Some(size) = styles.size {
            text_style.set_font_size(size.pixel_size(self.dpi));
        }
        let text_align = match styles.align {
            Alignment::Left => skia_safe::textlayout::TextAlign::Left,
            Alignment::Center => skia_safe::textlayout::TextAlign::Center,
            Alignment::Right => skia_safe::textlayout::TextAlign::Right,
            Alignment::Justify => skia_safe::textlayout::TextAlign::Justify,
        };

        if let Some(Foreground { color: fg_color }) = styles.foreground {
            let mut foreground_paint = Paint::new(
                Into::<Color4f>::into(self.resolve_color_ref(fg_color)?),
                None
            );
            foreground_paint.set_anti_alias(true);
            text_style.set_foreground_color(&foreground_paint);
        }

        if let Some(TextBackground { color: bg_color }) = styles.background {
            let mut background_paint = Paint::new(
                Into::<Color4f>::into(self.resolve_color_ref(bg_color)?),
                None
            );
            background_paint.set_anti_alias(true);
            text_style.set_background_color(&background_paint);
        }
    
        let mut paragraph_style = ParagraphStyle::new();
        paragraph_style.set_text_style(&text_style);
        paragraph_style.set_text_align(text_align);
        Ok(Some(paragraph_style))
    }
}
