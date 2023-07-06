pub mod model;
pub mod printing;
pub mod templates;

// re-export model objects
pub use model::{
    Layout,
    geometry::{ Geometry, Insets },
    elements::{
        Element, Frame,
        containers::Box,
        shapes::{ Background, Rectangle },
        text::Text,
    },
    styles::{
        PathStyle, TextStyle,
        fill::Fill,
        font::Font,
        only_if::{ OnlyIf, OnlyIfOperator },
        stroke::Stroke,
    },
};