use std::{str::FromStr, sync::OnceLock};

use handlebars::{Handlebars, Context};
use miette::IntoDiagnostic;

static HANDLEBARS: OnceLock<Handlebars> = OnceLock::new();

#[derive(PartialEq, Eq, Debug)]
pub enum TemplateAwareString {
    RawString(String),
    Template(String),
}

impl TemplateAwareString {
    pub fn new(contents: String) -> TemplateAwareString {
        if contents.contains("{{") {
            TemplateAwareString::Template(contents)
        } else {
            TemplateAwareString::RawString(contents)
        }
    }

    pub fn render(&self, ctx: &Context) -> miette::Result<String> {
        match self {
            Self::RawString(s) => Ok(s.clone()),
            Self::Template(tpl) => {
                let hb = HANDLEBARS.get_or_init(init_handlebars);
                Ok(hb.render_template_with_context(&tpl, ctx).into_diagnostic()?)
            }
        }
    }
}

impl FromStr for TemplateAwareString {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TemplateAwareString::new(s.to_string()))
    }
}

fn init_handlebars() -> Handlebars<'static> {
    let mut hb = Handlebars::new();
    hb.register_escape_fn(handlebars::no_escape);
    hb
}