use std::{collections::HashMap, path::Path, fs};

use miette::{Diagnostic, IntoDiagnostic};
use thiserror::Error;

use crate::{layout::{Layout, model::styles::color::Color}, config::{sheets::layout::Sheet, RawConfig}};

use super::{globals, card::{Card, self}};

pub struct Project {
    cards: HashMap<String, Card>,
    layouts: HashMap<String, Layout>,
    colors: HashMap<String, Color>,
    sheet_layouts: HashMap<String, Sheet>,
    pub pdf_metadata: PdfMetadata,
}

impl Project {
    pub fn new() -> Project {
        Project {
            cards: HashMap::new(),
            layouts: HashMap::new(),
            colors: HashMap::new(),
            sheet_layouts: HashMap::new(),
            pdf_metadata: PdfMetadata::default(),
        }
    }

    pub fn load_from_directory<P: AsRef<Path>>(project_dir: P) -> miette::Result<Project> {
        if project_dir.as_ref().is_dir() {
            log::info!("Loading project from directory {}", project_dir.as_ref().to_str().unwrap_or_default());
            let mut project = Project::new();
            for entry in fs::read_dir(project_dir).into_diagnostic()? {
                let entry = entry.into_diagnostic()?;
                let path = entry.path();
                match path.extension().and_then(|p| p.to_str()) {
                    Some("layout") => {
                        let file_contents_bytes = fs::read(&path).into_diagnostic()?;
                        let file_contents_str = std::str::from_utf8(file_contents_bytes.as_slice()).into_diagnostic()?;
                        let file_name =
                            path.file_name()
                                .and_then(|p| p.to_str())
                                .ok_or(ProjectConfigurationError::Other(format!("Path {} has no filename", path.display())))?;
                        let file_stem =
                            path.file_stem()
                                .and_then(|p| p.to_str())
                                .ok_or(ProjectConfigurationError::Other(format!("Path {} has no file stem", path.display())))?;
                        let layout: Layout = knuffel::parse(file_name, file_contents_str)?;
                        project.register_layout(file_stem, layout);
                        log::info!("Successfully loaded layout \"{}\" from file {}", file_stem, file_name);
                    },
                    Some("csv") => {
                        let set_name = path.file_stem().and_then(|p| p.to_str()).unwrap_or_default();
                        log::info!("Found card set \"{}\" in file {}",
                            set_name,
                            path.file_name().and_then(|p| p.to_str()).unwrap_or_default(),
                        );
                        let csv_cards = card::loaders::load_csv(&path)?;
                        for card in csv_cards {
                            log::debug!("Loaded card \"{}\" from card set \"{}\"", card.id, set_name);
                            project.add_card(card);
                        }
                    },
                    Some("xls") | Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("ods") => {
                        let set_name = path.file_stem().and_then(|p| p.to_str()).unwrap_or_default();
                        log::info!("Found card set \"{}\" in file {}",
                            set_name,
                            path.file_name().and_then(|p| p.to_str()).unwrap_or_default(),
                        );
                        let xls_cards = card::loaders::load_excel(&path)?;
                        for card in xls_cards {
                            log::debug!("Loaded card \"{}\" from card set \"{}\"", card.id, set_name);
                            project.add_card(card);
                        }
                    },
                    Some("conf") => {
                        let file_contents_bytes = fs::read(&path).into_diagnostic()?;
                        let file_contents_str = std::str::from_utf8(file_contents_bytes.as_slice()).into_diagnostic()?;
                        let file_name =
                            path.file_name()
                                .and_then(|p| p.to_str())
                                .ok_or(ProjectConfigurationError::Other(format!("Path {} has no filename", path.display())))?;
                        let config: RawConfig = knuffel::parse(file_name, file_contents_str)?;
                        let new_colors = config.get_colors()?;
                        let new_color_count = new_colors.len();
                        let new_sheet_layouts = config.get_sheet_layouts()?;
                        let new_sheet_layout_count = new_sheet_layouts.len();
                        project.colors.extend(new_colors);
                        project.sheet_layouts.extend(new_sheet_layouts);

                        project.pdf_metadata.author = config.pdf_author;
                        project.pdf_metadata.title = config.pdf_title;
                        project.pdf_metadata.subject = config.pdf_subject;
                        project.pdf_metadata.keywords = config.pdf_keywords;

                        log::info!("Successfully loaded {} colors and {} sheet layouts from file {}", new_color_count, new_sheet_layout_count, file_name);
                    },
                    _ => {},
                }
            }

            log::info!("Finished loading project");
            Ok(project)
        } else {
            Err(ProjectConfigurationError::NotADirectory(
                format!("{}",project_dir.as_ref().display())
            ).into())
        }
    }

    pub fn card_by_id(&self, id: &str) -> Result<&Card, ProjectConfigurationError> {
        self.cards.get(id)
            .ok_or_else(|| ProjectConfigurationError::NoSuchCard(id.to_string()))
    }

    pub fn all_cards(&self) -> impl Iterator<Item = &Card> {
        self.cards.values()
    }

    pub fn add_card(&mut self, card: Card) -> () {
        self.cards.insert(card.id.clone(), card);
    }

    pub fn layout_for_card(&self, card: &Card) -> Result<&Layout, ProjectConfigurationError> {
        self.layout_named(
            &card.layout_name().unwrap_or("default".to_string())
        ).ok_or_else(|| ProjectConfigurationError::NoLayoutFound(card.id.clone()))
    }

    pub fn color_named(&self, name: &str) -> Result<Color, ProjectConfigurationError> {
        self.colors
            .get(name)
            .map(|c| c.clone())
            .or_else(||globals::color_named(name))
            .ok_or_else(|| ProjectConfigurationError::InvalidColorName(name.to_string()))
    }

    pub fn layout_named(&self, name: &str) -> Option<&Layout> {
        self.layouts
            .get(name)
            .or_else(|| globals::layout_named(name))
    }

    pub fn register_layout(&mut self, name: &str, layout: Layout) -> () {
        self.layouts.insert(name.to_string(), layout);
    }

    pub fn sheet_type_named(&self, name: &str) -> Option<&Sheet> {
        self.sheet_layouts.get(name)
    }
}

pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
}

impl Default for PdfMetadata {
    fn default() -> Self {
        PdfMetadata { title: None, author: None, subject: None, keywords: None }
    }
}

#[derive(Error, Diagnostic, Debug)]
pub enum ProjectConfigurationError {
    #[error("no card exists with id '{0}'")]
    NoSuchCard(String),
    #[error("couldn't find a layout for card with id '{0}' (check that its layout property is correct)")]
    NoLayoutFound(String),
    #[error("couldn't find a definition for a color named '{0}'")]
    InvalidColorName(String),
    #[error("project path {0} is not a directory")]
    NotADirectory(String),
    #[error("{0}")]
    Other(String),
}
