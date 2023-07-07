use std::{str::FromStr, sync::OnceLock, error::Error};

use handlebars::{Handlebars, Context};
use miette::{SourceOffset, Diagnostic};
use thiserror::Error;

static HANDLEBARS: OnceLock<Handlebars> = OnceLock::new();

#[derive(PartialEq, Eq, Debug, Clone)]
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

    pub fn render(&self, ctx: &Context) -> Result<String, TemplateError> {
        match self {
            Self::RawString(s) => Ok(s.clone()),
            Self::Template(tpl) => {
                let hb = HANDLEBARS.get_or_init(init_handlebars);
                let err_helper = ErrorHelper::of(tpl.as_str());
                hb
                    .render_template_with_context(&tpl, ctx)
                    .map_err(|err| err_helper.from_hb(err))
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

#[derive(Error, Diagnostic, Debug)]
pub enum TemplateError {
    #[error(transparent)]
    UnspecifiedError(#[from] handlebars::RenderError),
    #[error("error while filling in template: {description}")]
    TemplateErrorAtLocation {
        description: String,
        #[source_code]
        template_source: String,
        #[label("error occurred here")]
        offset: SourceOffset,
    },
}

struct ErrorHelper<'a> {
    template_source: &'a str
}

impl ErrorHelper<'_> {
    fn of(template_source: &str) -> ErrorHelper {
        ErrorHelper { template_source }
    }

    fn from_hb(&self, render_err: handlebars::RenderError) -> TemplateError {
        match render_err.source() {
            Some(inner) => match inner.downcast_ref::<handlebars::TemplateError>() {
                Some(template_err) => match (template_err.line_no, template_err.column_no) {
                    (Some(line), Some(column)) =>
                        TemplateError::TemplateErrorAtLocation {
                            description: format!("{}", template_err.reason()),
                            template_source: self.template_source.to_string(),
                            offset: SourceOffset::from_location(
                                self.template_source,
                                line,
                                column
                            ),
                        },
                    _ => TemplateError::UnspecifiedError(render_err),
                },
                None => TemplateError::UnspecifiedError(render_err),
            },
            None => TemplateError::UnspecifiedError(render_err),
        }
    }
}
