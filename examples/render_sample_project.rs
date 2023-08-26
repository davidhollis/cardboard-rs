use std::collections::HashMap;

use cardboard::{data::project::Project, renderer::{SkiaRenderer, Renderer}};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{append::console::ConsoleAppender, encode::pattern::PatternEncoder, Config, config::{Appender, Root, Logger}};
use miette::IntoDiagnostic;
use regex::Regex;

const BASE_DIR: &str = env!("CARGO_MANIFEST_DIR");

lazy_static! {
    static ref NON_IDENTIFIER_SEQUENCE: Regex = Regex::new(r#"[^A-Za-z0-9_]+"#).unwrap();
    static ref DECKLIST_LINE: Regex = Regex::new(r#"\A(?<quantity>[0-9]+) (?<id>.+)\z"#).unwrap();
}

const SAMPLE_DECK: &[(usize, &'static str)] = &[
    (1, "alpha::0001"),
    (2, "alpha::0002"),
    (3, "alpha_0003"),
    (4, "alpha_0004"),
    (5, "beta_0003"),
];

fn main() -> miette::Result<()> {
    let verbose = std::env::args().find(|arg| (arg == "-v" || arg == "--verbose")).is_some();
    init_logger(verbose)?;

    let sample_project_dir = format!("{}/examples/projects/sample", BASE_DIR);
    let output_dir = format!("{}/test-renders/sample-project", BASE_DIR);
    std::fs::create_dir_all(&output_dir).into_diagnostic()?;

    let sample_project = Project::load_from_directory(sample_project_dir)?;
    let mut renderer = SkiaRenderer::new();
    let mut rendered_cards: HashMap<String, <SkiaRenderer as Renderer>::SingleCard<'_>> = HashMap::new();

    // Render each card and write it out as a png
    for card in sample_project.all_cards() {
        let sanitized_card_id = NON_IDENTIFIER_SEQUENCE.replace_all(&card.id, "_").to_string();
        let card_image_path = format!("{}/{}.png", output_dir, sanitized_card_id);

        log::info!("Rendering card \"{}\" to {}", card.id, card_image_path);

        let rendered_card = renderer.render_single(&sample_project, &card.id)?;
        renderer.write_png(&rendered_card, card_image_path)?;
        rendered_cards.insert(card.id.clone(), rendered_card);
    }

    // Build a "deck" and lay it out as a set of sheets in a pdf
    log::info!("Building a sample deck");
    let mut deck = vec![];
    for (count, card_id) in SAMPLE_DECK {
        for _ in 0..*count {
            deck.push(rendered_cards.get(*card_id).ok_or(miette::miette!("Missing card {} for some reason", card_id))?);
        }
    }
    let deck_path = format!("{}/all_sample_cards.pdf", output_dir);
    log::info!("Writing sheets to pdf file at {}", deck_path);
    renderer.write_deck_pdf(deck.into_iter(), deck_path, sample_project.sheet_type_named("default").unwrap())?;

    log::info!("Finished rendering {} cards and 1 deck", sample_project.all_cards().count());

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
