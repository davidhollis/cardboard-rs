use std::path::Path;

use cardboard::{data::Card, layout::Layout, renderer::{SkiaRenderer, Renderer}};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const TEST_CARD_LAYOUT: &str = r###"
geometry {
    width 825
    height 1125
    cut 37
    safe 75
}
background {
    solid "white"
}
rectangle x=100 y=100 w=625 h=925 {
    stroke 5 "black" {
        pattern "dotted"
    }
    solid "rgb(128, 255, 128)"
}
text "Hello from {{name}}

version {{version}}" {
    frame x=150 y=200 w=525 h=725
    font family="Optima" weight="bold"
    size 10 "pt"
    foreground "rgb(64, 64, 192)"
}
"###;

fn main() -> miette::Result<()> {
    println!("Loading test layout...");
    let layout: Layout = knuffel::parse("test.kdl", TEST_CARD_LAYOUT)?;

    println!("Setting up test card...");
    let mut test_card = Card::new("testing".to_string());
    test_card.fields_mut().insert("name".to_string(), NAME.to_string());
    test_card.fields_mut().insert("version".to_string(), VERSION.to_string());

    println!("Initializing Skia rendering engine...");
    let skia = SkiaRenderer::new();

    println!("Rendering test card...");
    let test_card_image = skia.render_single(&test_card, &layout)?;

    println!("Saving test card image to ./test-card.png...");
    skia.write_png(test_card_image, Path::new("./test-card.png"))?;

    println!("Done!");

    Ok(())
}
