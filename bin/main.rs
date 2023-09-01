use std::{path::{PathBuf, Path}, collections::HashMap, io::{BufReader, BufRead}, fs::File, iter::{repeat}};

use cardboard::{data::{globals, project::Project, card::Card}, renderer::{SkiaRenderer, Renderer}};
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{append::console::ConsoleAppender, encode::pattern::PatternEncoder, Config, config::{Appender, Logger, Root}};
use miette::IntoDiagnostic;
use regex::Regex;


lazy_static! {
    static ref NON_IDENTIFIER_SEQUENCE: Regex = Regex::new(r#"[^A-Za-z0-9_]+"#).unwrap();
    static ref DECKLIST_LINE: Regex = Regex::new(r#"\A([0-9]+)\s+(.+)\z"#).unwrap();
}


/// Compile a cardboard project into a set of card images or pdf sheets.
#[derive(Parser)]
#[command(author, version, long_about = None)]
struct Cardboard {
    /// The directory the project should be loaded from (default: the current directory).
    #[arg(value_name = "DIR")]
    project_dir: Option<PathBuf>,

    /// Show more detailed output.
    #[arg(long, short)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Clone)]
enum Command {
    /// Render a list of cards in a PDF, organized on printable sheets.
    /// 
    /// The PDF is written to the specified path if given, or to
    /// `<PROJECT_DIR>/_output/<SOMETHING>.pdf` if not, where SOMETHING is
    /// derived from the name of the card list file or the sheet layout, or
    /// just "sheet" if none of the above were given.
    /// 
    /// The cards will be laid out in the same order as they appear in the
    /// list. If there is no card list given, every card in the project will be
    /// included in the PDF, in an arbitrary order.
    Sheet {
        /// The name of the sheet layout.
        #[arg(short, long, value_name = "TYPE")]
        sheet_type: Option<String>,

        /// A path to a list of cards to include (.deck format). If the path
        /// is "-", it'll be read from stdin.
        #[arg(short, long, value_name = "PATH")]
        card_list: Option<PathBuf>,

        /// The path to the finished PDF.
        #[arg(short, long, value_name = "PATH")]
        output_path: Option<PathBuf>,
    },

    /// Render a set of cards as individual PNG images.
    /// 
    /// By default, this renders every card in all sets in the project. If one
    /// or more sets (`-s <SET_ID>`) or cards (`-c <CARD_ID>`) are specified,
    /// only those cards and sets are rendered.
    /// 
    /// The card images are written in the specified output directory, or to
    /// `<PROJECT_DIR>/_output` if none was given. Each card will be named
    /// `<CARD_ID>.png`.
    Singles {
        /// Render the specified set.
        #[arg(short = 's', long = "set", value_name = "SET_ID")]
        sets: Vec<String>,

        /// Render the specified card.
        #[arg(short = 'c', long = "card", value_name = "CARD_ID")]
        cards: Vec<String>,

        /// The directory that all the card images should be written to.
        #[arg(short, long, value_name = "DIR")]
        output_dir: Option<PathBuf>,
    }
}

impl Default for Command {
    fn default() -> Self {
        Command::Sheet { sheet_type: None, card_list: None, output_path: None }
    }
}

fn main() -> miette::Result<()> {
    let cb = Cardboard::parse();
    init_logger(cb.verbose)?;
    globals::init_global_data()?;

    let project_dir =
        cb.project_dir
            .unwrap_or(std::env::current_dir().into_diagnostic()?)
            .canonicalize().into_diagnostic()?;
    let project = Project::load_from_directory(&project_dir)?;
    let mut renderer = SkiaRenderer::new();

    match cb.command.unwrap_or_default() {
        Command::Sheet { sheet_type, card_list, output_path } => {
            let output_path = output_path.unwrap_or_else(|| {
                let default_output_path = project_dir.join("_output");
                let stem = match card_list.as_ref().and_then(|cl| cl.file_stem()).and_then(|s| s.to_str()) {
                    Some("-") | None =>
                        sheet_type.as_ref()
                            .map(|st| st.as_str())
                            .unwrap_or("sheet"),
                    Some(s) => s,
                };
                default_output_path.join(format!("{stem}.pdf"))
            });
            output_path.parent().map(std::fs::create_dir_all).unwrap_or(Ok(())).into_diagnostic()?;

            let sheet_name =
                sheet_type.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("default");
            let sheet =
                project.sheet_type_named(sheet_name)
                    .ok_or_else(|| miette::miette!("Couldn't find sheet layout named \"{}\"", sheet_name))?;

            let card_ids = match card_list {
                Some(card_list) => load_card_list(&card_list)?,
                None => project.all_cards().map(|c| c.id.clone()).collect()
            };
            let selected_cards = project.all_cards().filter(|c| card_ids.contains(&c.id));
            let mut rendered_cards: HashMap<String, <SkiaRenderer as Renderer>::SingleCard<'_>> = HashMap::new();

            for card in selected_cards {
                log::info!("Rendering card \"{}\"", card.id);
                let rendered_card = renderer.render_single(&project, &card.id)?;
                rendered_cards.insert(card.id.clone(), rendered_card);
            }

            renderer.write_deck_pdf(
                card_ids.iter().flat_map(|id| rendered_cards.get(id).into_iter()),
                &output_path,
                sheet
            )?;

            log::info!("Successfully wrote pdf {}", output_path.display());
        },
        Command::Singles { sets, cards, output_dir } => {
            let output_dir = output_dir.unwrap_or(project_dir.join("_output"));
            std::fs::create_dir_all(&output_dir).into_diagnostic()?;
            let selected_cards: Box<dyn Iterator<Item = &Card>> =
                if sets.is_empty() && cards.is_empty() {
                    Box::new(project.all_cards())
                } else {
                    Box::new(project
                        .all_cards()
                        .filter(|card| sets.contains(&card.set) || cards.contains(&card.id)))
                };
            
            let mut i = 0usize;
            for card in selected_cards {
                let sanitized_card_id = NON_IDENTIFIER_SEQUENCE.replace_all(&card.id, "_").to_string();
                let card_image_path = output_dir.join(format!("{sanitized_card_id}.png"));

                log::info!("Rendering card \"{}\" to {}", card.id, card_image_path.strip_prefix(&project_dir).unwrap_or(&card_image_path.to_owned()).display());

                let rendered_card = renderer.render_single(&project, &card.id)?;
                renderer.write_png(&rendered_card, card_image_path)?;
                i += 1;
            }

            log::info!("Successfully rendered {} cards!", i);
        }
    }

    Ok(())
}

fn load_card_list<P: AsRef<Path>>(card_list_path: P) -> miette::Result<Vec<String>> {
    let mut ids = vec![];
    let card_list_file: Box<dyn BufRead> =
        if card_list_path.as_ref().display().to_string() == "-" {
            Box::new(std::io::stdin().lock())
        } else {
            Box::new(BufReader::new(File::open(card_list_path).into_diagnostic()?))
        };
    for line in card_list_file.lines() {
        let line = line.into_diagnostic()?;
        let captures = DECKLIST_LINE.captures(&line).map(|c| c.extract());
        match captures {
            Some((_, [qty_str, id])) => {
                let qty: usize = qty_str.parse().into_diagnostic()?;
                ids.extend(repeat(id.to_string()).take(qty));
            },
            None => {
                ids.push(line);
            }
        }
    }
    
    Ok(ids)
}

fn init_logger(verbose: bool) -> miette::Result<()> {
    let stdout_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("[{h({l:>7})}] {m}{n}")))
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
