use std::str::FromStr;

use miette::miette;

use crate::layout::templates::TemplateAwareString;

#[derive(knuffel::Decode, PartialEq, Eq, Debug, Clone)]
pub struct OnlyIf {
    #[knuffel(argument, str)]
    pub left: TemplateAwareString,
    #[knuffel(argument, str)]
    pub op: Option<OnlyIfOperator>,
    #[knuffel(arguments, str)]
    pub right: Vec<TemplateAwareString>,
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
