use crate::Renderer;

use cairo::{ Context, Format, ImageSurface };
use std::fs::File;
use std::io::Write;
use std::path::Path;

use layout::prelude::*;

const ANGLE_LEFT: f64   = std::f64::consts::PI;
const ANGLE_TOP: f64    = 3.0 * std::f64::consts::FRAC_PI_2;
const ANGLE_RIGHT: f64  = 0.0;
const ANGLE_BOTTOM: f64 = std::f64::consts::FRAC_PI_2;

pub struct CairoRenderer {
    layout: Layout,
}

impl CairoRenderer {
    pub fn new(layout: Layout) -> CairoRenderer { CairoRenderer { layout } }
}

impl Renderer for CairoRenderer {
    fn render_png_to(&self, _context: Box<dyn content::Context>, path: &Path) -> anyhow::Result<()> {
        let surface = ImageSurface::create(
            Format::ARgb32,
            self.layout.geometry.width.into(),
            self.layout.geometry.height.into()
        ).coerce_result()?;

        let context = Context::new(&surface).coerce_result()?;

        for element in self.layout.elements.iter() {
            context.render_element(&element)?;
        }

        let mut out_file = File::create(path).coerce_result()?;
        surface.write_to_png(&mut out_file).coerce_result()?;
        out_file.flush().coerce_result()?;
        out_file.sync_all().coerce_result()?;

        Ok(())
    }
}

trait CairoContextExtn {
    fn render_element(&self, el: &Element) -> anyhow::Result<()>;
    fn trace_shape(&self, shape: &Shape) -> anyhow::Result<()>;
    fn setup_stroke(&self, stroke: &Stroke) -> anyhow::Result<()>;
    fn setup_fill(&self, fill: &Fill) -> anyhow::Result<()>;
    fn setup_color(&self, color: &Color) -> anyhow::Result<()>;
}

impl CairoContextExtn for Context {
    fn render_element(&self, el: &Element) -> anyhow::Result<()> {
        match el {
            Element::Shape(shape, stroke, fill) => {
                self.save().coerce_result()?;
                self.trace_shape(shape)?;
                self.setup_stroke(stroke)?;
                self.stroke_preserve().coerce_result()?;
                self.setup_fill(fill)?;
                self.fill().coerce_result()?;
                self.restore().coerce_result()?;
            },
        }
        Ok(())
    }

    fn trace_shape(&self, shape: &Shape) -> anyhow::Result<()> {
        match shape {
            Shape::Rectangle(Rectangle { x, y, width, height, corner_radius }) => {
                if *corner_radius == 0 {
                    // regular old rectangle
                    self.rectangle(
                        f64::from(*x),
                        f64::from(*y),
                        f64::from(*width),
                        f64::from(*height)
                    )
                } else {
                    // rounded rectangle
                    let top = f64::from(*y);
                    let bottom = top + f64::from(*height);
                    let left = f64::from(*x);
                    let right = left + f64::from(*width);
                    let r = f64::from(*corner_radius);

                    self.arc(left + r,  top + r,    r, ANGLE_LEFT,   ANGLE_TOP);
                    self.arc(right - r, top + r,    r, ANGLE_TOP,    ANGLE_RIGHT);
                    self.arc(right - r, bottom - r, r, ANGLE_RIGHT,  ANGLE_BOTTOM);
                    self.arc(left + r,  bottom - r, r, ANGLE_BOTTOM, ANGLE_LEFT);
                    self.close_path();
                }
            }
        }

        Ok(())
    }

    fn setup_stroke(&self, stroke: &Stroke) -> anyhow::Result<()> {
        match stroke {
            Stroke { stroke_type: StrokeType::None, .. } => {
                self.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                self.set_line_width(0.0);
            }
            Stroke { stroke_type, color, width } => {
                self.setup_color(color)?;
                self.set_line_width(f64::from(*width));
                match stroke_type {
                    StrokeType::Dashed => self.set_dash(&[20.0, 20.0], 0.0),
                    StrokeType::Dotted => self.set_dash(&[2.0, 2.0], 0.0),
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn setup_fill(&self, fill: &Fill) -> anyhow::Result<()> {
        match fill {
            Fill::None => self.set_source_rgba(0.0, 0.0, 0.0, 0.0),
            Fill::Solid(color) => self.setup_color(color)?,
        }
        Ok(())
    }

    fn setup_color(&self, color: &Color) -> anyhow::Result<()> {
        match color {
            Color::RGBA { red, blue, green, alpha } => {
                self.set_source_rgba(
                    f64::from(*red) / 255.0,
                    f64::from(*green) / 255.0,
                    f64::from(*blue) / 255.0,
                    f64::from(*alpha) / 255.0,
                )
            },
            Color::Named { .. } => return Err(anyhow!("Named colors are not yet implemented")),
        }
        Ok(())
    }
}

trait UniformResultType<T> {
    fn coerce_result(self) -> anyhow::Result<T>;
}

impl<T> UniformResultType<T> for cairo::Result<T> {
    fn coerce_result(self) -> anyhow::Result<T> {
        self.map_err(|err| anyhow!("Cairo error: {}", err))
    }
}

impl<T> UniformResultType<T> for std::io::Result<T> {
    fn coerce_result(self) -> anyhow::Result<T> {
        self.map_err(|err| anyhow!("IO error: {}", err))
    }
}

impl<T> UniformResultType<T> for std::result::Result<T, cairo::IoError> {
    fn coerce_result(self) -> anyhow::Result<T> {
        self.map_err(|err| anyhow!("Cairo IO error: {}", err))
    }
}