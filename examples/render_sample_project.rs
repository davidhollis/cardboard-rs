use cardboard::{data::project::Project, renderer::{SkiaRenderer, Renderer}};
use lazy_static::lazy_static;
use miette::IntoDiagnostic;
use regex::Regex;

const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

lazy_static! {
    static ref NON_IDENTIFIER_SEQUENCE: Regex = Regex::new(r#"[^A-Za-z0-9_]+"#).unwrap();
}

fn main() -> miette::Result<()> {
    let sample_project_dir = format!("{}/examples/projects/sample", BASE_DIR);
    let output_dir = format!("{}/test-renders/sample-project", BASE_DIR);
    std::fs::create_dir_all(&output_dir).into_diagnostic()?;

    let sample_project = Project::load_from_directory(sample_project_dir)?;
    let renderer = SkiaRenderer::new();

    for card in sample_project.all_cards() {
        let rendered_card = renderer.render_single(&sample_project, &card.id)?;
        let sanitized_card_id = NON_IDENTIFIER_SEQUENCE.replace_all(&card.id, "_").to_string();
        renderer.write_png(rendered_card, format!("{}/{}.png", output_dir, sanitized_card_id))?;
    }

    Ok(())
}
