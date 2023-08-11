use std::collections::HashMap;

use miette::Diagnostic;
use thiserror::Error;

use crate::layout::{Layout, model::styles::color::Color};

use super::{config::Config, globals, card::Card};

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
}
