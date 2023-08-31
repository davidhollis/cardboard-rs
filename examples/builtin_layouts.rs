use std::collections::HashMap;

use cardboard::{data::{project::Project, globals}, renderer::{SkiaRenderer, Renderer}};
use log::LevelFilter;
use log4rs::{append::console::ConsoleAppender, encode::pattern::PatternEncoder, Config, config::{Appender, Root, Logger}};
use miette::IntoDiagnostic;

const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> miette::Result<()> {
    let verbose = std::env::args().find(|arg| (arg == "-v" || arg == "--verbose")).is_some();
    init_logger(verbose)?;

    globals::init_global_data()?;

    let project_dir = format!("{}/examples/projects/builtin-layouts", BASE_DIR);
    let output_dir = format!("{}/test-renders/builtin-layouts", BASE_DIR);
    std::fs::create_dir_all(&output_dir).into_diagnostic()?;

    let project = Project::load_from_directory(project_dir)?;
    let mut renderer = SkiaRenderer::new();
    let mut rendered_cards: HashMap<String, <SkiaRenderer as Renderer>::SingleCard<'_>> = HashMap::new();

    // Render each card and write it out as a png
    for card in project.all_cards() {
        let card_image_path = format!("{}/{}.png", output_dir, card.id);

        log::info!("Rendering card \"{}\" to {}", card.id, card_image_path);

        let rendered_card = renderer.render_single(&project, &card.id)?;
        renderer.write_png(&rendered_card, card_image_path)?;
        rendered_cards.insert(card.id.clone(), rendered_card);
    }

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
