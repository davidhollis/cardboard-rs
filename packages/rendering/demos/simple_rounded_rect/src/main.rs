use std::env::args;
use std::path::Path;

use content::EmptyContext;
use layout::prelude::*;
use rendering::Renderer;
use rendering::cairo::CairoRenderer;

fn build_simple_layout() -> Layout {
    Layout {
        geometry: Geometry {
            height: 1125,
            width: 825,
            cut: Insets::uniform(37),
            safe: Insets::uniform(75),
            units: Units::Pixels,
        },
        background: Fill::Solid(Color::RGBA { red: 0, green: 0, blue: 255, alpha: 128 }),
        elements: vec![
            Element::Shape(
                Shape::Rectangle(Rectangle {
                    x: 212,
                    y: 362,
                    width: 400,
                    height: 400,
                    corner_radius: 50,
                }),
                Stroke {
                    stroke_type: StrokeType::Dashed,
                    color: Color::RGBA { red: 0, green: 0, blue: 64, alpha: 255 },
                    width: 10,
                },
                Fill::Solid(Color::RGBA { red: 0, green: 0, blue: 255, alpha: 255 }),
            )
        ],
    }
}

fn main() {
    let filename = args().nth(1).unwrap();
    let path = Path::new(&filename);

    let renderer = CairoRenderer::new(build_simple_layout());
    renderer.render_png_to(Box::new(EmptyContext {}), path).unwrap();

    println!("Wrote image to: {}", path.to_str().unwrap());
}
