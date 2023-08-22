use cardboard::{data::project::Project, renderer::{SkiaRenderer, Renderer}};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{append::console::ConsoleAppender, encode::pattern::PatternEncoder, Config, config::{Appender, Root, Logger}};
use miette::IntoDiagnostic;
use regex::Regex;

const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

lazy_static! {
    static ref NON_IDENTIFIER_SEQUENCE: Regex = Regex::new(r#"[^A-Za-z0-9_]+"#).unwrap();
}

fn main() -> miette::Result<()> {
    let verbose = std::env::args().find(|arg| (arg == "-v" || arg == "--verbose")).is_some();
    init_logger(verbose)?;

    let sample_project_dir = format!("{}/examples/projects/sample", BASE_DIR);
    let output_dir = format!("{}/test-renders/sample-project", BASE_DIR);
    std::fs::create_dir_all(&output_dir).into_diagnostic()?;

    let sample_project = Project::load_from_directory(sample_project_dir)?;
    let renderer = SkiaRenderer::new();

    for card in sample_project.all_cards() {
        let sanitized_card_id = NON_IDENTIFIER_SEQUENCE.replace_all(&card.id, "_").to_string();
        let card_image_path = format!("{}/{}.png", output_dir, sanitized_card_id);

        log::info!("Rendering card \"{}\" to {}", card.id, card_image_path);

        let rendered_card = renderer.render_single(&sample_project, &card.id)?;
        renderer.write_png(rendered_card, card_image_path)?;
    }

    log::info!("Finished rendering {} cards", sample_project.all_cards().count());

    Ok(())
}

fn init_logger(verbose: bool) -> miette::Result<()> {
    let stdout_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{h({l:>7})}] {t} - {m}{n}")))
        .build();
    let log4rs_config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout_appender)))
        .logger(Logger::builder()
            .appender("stdout")
            .additive(false)
            .build("cardboard", if verbose { LevelFilter::Debug } else { LevelFilter::Info })
        )
        .build(Root::builder()
            .appender("stdout")
            .build(LevelFilter::Info)
        )
        .into_diagnostic()?;
    let _ = log4rs::init_config(log4rs_config).into_diagnostic()?;
    Ok(())
}
