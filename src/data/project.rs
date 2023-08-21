use std::{collections::HashMap, path::Path, fs};

use miette::{Diagnostic, IntoDiagnostic};
use thiserror::Error;

use crate::layout::{Layout, model::styles::color::Color};

use super::{config::Config, globals, card::{Card, self}};

pub struct Project {
    cards: HashMap<String, Card>,
    layouts: HashMap<String, Layout>,
    colors: HashMap<String, Color>,
    config: Option<Config>,
}

impl Project {
    pub fn new() -> Project {
        Project {
            cards: HashMap::new(),
            layouts: HashMap::new(),
            colors: HashMap::new(),
            config: None,
        }
    }

    pub fn load_from_directory<P: AsRef<Path>>(project_dir: P) -> miette::Result<Project> {
        if project_dir.as_ref().is_dir() {
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
                    },
                    Some("csv") => {
                        let csv_cards = card::loaders::load_csv(path)?;
                        for card in csv_cards {
                            project.add_card(card);
                        }
                    },
                    // Some("xls") | Some("xlsx") => { /* load a bunch of cards from an excel file */ },
                    // Some("colors") => { /* load project colors */ },
                    // Some("conf") => { /* load project config */ },
                    _ => {},
                }
            }
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

    pub fn config(&self) -> &Config {
        self.config.as_ref().unwrap_or_else(|| globals::default_config())
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
