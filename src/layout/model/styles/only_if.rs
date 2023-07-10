use std::str::FromStr;

use handlebars::Context;
use miette::miette;

use crate::layout::templates::{TemplateAwareString, TemplateError};

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct OnlyIf {
    #[knuffel(argument, str)]
    pub left: TemplateAwareString,
    #[knuffel(argument, str)]
    pub op: Option<OnlyIfOperator>,
    #[knuffel(arguments, str)]
    pub right: Vec<TemplateAwareString>,
}

impl OnlyIf {
    pub fn evaluate(&self, ctx: &Context) -> Result<bool, miette::Error> {
        let left_val = self.left.render(ctx)?;
        let right_vals: Vec<String> = self.right.iter().map(|tpl| tpl.render(ctx)).into_iter().collect::<Result<Vec<String>,TemplateError>>()?;

        match self.op {
            Some(OnlyIfOperator::Equal) => {
                if let Some(right_val) = right_vals.first() {
                    Ok(&left_val == right_val)
                } else {
                    Ok(false)
                }
            },
            Some(OnlyIfOperator::NotEqual) => {
                if let Some(right_val) = right_vals.first() {
                    Ok(&left_val != right_val)
                } else {
                    Ok(false)
                }
            },
            Some(OnlyIfOperator::In) => Ok(right_vals.contains(&left_val)),
            Some(OnlyIfOperator::NotIn) => Ok(!right_vals.contains(&left_val)),
            None => Ok(!left_val.is_empty()),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum OnlyIfOperator {
    Equal,
    NotEqual,
    In,
    NotIn,
}

impl FromStr for OnlyIfOperator {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" => Ok(OnlyIfOperator::Equal),
            "!=" => Ok(OnlyIfOperator::NotEqual),
            "in" => Ok(OnlyIfOperator::In),
            "not in" => Ok(OnlyIfOperator::NotIn),
            _ => Err(miette!(r#"Invalid only-if operator. Expected one of `"="`, `"!="`, `"in"`, or `"not in"`."#))
        }
    }
}
