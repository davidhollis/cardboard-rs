use std::str::FromStr;

#[derive(PartialEq, Eq, Debug)]
pub enum TemplateAwareString {
    RawString(String),
    RegisteredTemplate { id: String }
}

impl TemplateAwareString {
    pub fn new(contents: String) -> TemplateAwareString {
        if contents.contains("{{") {
            let id = register_template(contents);
            TemplateAwareString::RegisteredTemplate { id }
        } else {
            TemplateAwareString::RawString(contents)
        }
    }
}

impl FromStr for TemplateAwareString {
    type Err = miette::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TemplateAwareString::new(s.to_string()))
    }
}

fn register_template(_contents: String) -> String {
    // TODO add a global handlebars instance
    todo!()
}