use std::path::Path;

use cardboard::{data::{card::Card, project::Project}, layout::model::Layout, renderer::{SkiaRenderer, Renderer}};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

const TEST_CARD_LAYOUT: &str = r###"
geometry {
    width 825
    height 1125
    cut 37
    safe 75
}
base {
    text {
        font family="Optima" weight="normal"
        align "center"
        size 8 "pt"
    }
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
text "Hello from {{name}}" {
    frame x=150 y=200 w=525 h=100
    font weight="black"
    size 11 "pt"
    foreground "rgb(64, 64, 192)"
}

text "version {{version}}" {
    frame x=150 y=300 w=525 h=625
}
"###;

fn main() -> miette::Result<()> {
    let mut test_project = Project::new();

    println!("Loading test layout...");
    let layout: Layout = knuffel::parse("test.kdl", TEST_CARD_LAYOUT)?;
    test_project.register_layout("test_layout", layout);

    println!("Setting project metadata...");
    test_project.pdf_metadata.author = Some("David Hollis <david@hollis.computer>".to_string());
    test_project.pdf_metadata.title = Some("Test Version Card".to_string());

    println!("Setting up test card...");
    let mut test_card = Card::new("test_card_id".to_string(), "version".to_string());
    test_card.fields_mut().insert("name".to_string(), NAME.to_string());
    test_card.fields_mut().insert("version".to_string(), VERSION.to_string());
    test_card.fields_mut().insert("layout".to_string(), "test_layout".to_string());
    test_project.add_card(test_card);

    println!("Initializing Skia rendering engine...");
    let mut skia = SkiaRenderer::new();

    println!("Rendering test card...");
    let test_card_image = skia.render_single(&test_project, "test_card_id")?;

    println!("Saving test card image to test-renders/version-card.png...");
    skia.write_png(&test_card_image, Path::new(format!("{}/test-renders/version-card.png", BASE_DIR).as_str()))?;

    println!("Saving test card pdf to test-renders/version-card.pdf...");
    skia.write_single_pdf(&test_card_image, Path::new(format!("{}/test-renders/version-card.pdf", BASE_DIR).as_str()))?;

    println!("Done!");

    Ok(())
}
