mod parser;

#[derive(PartialEq, Eq, Debug)]
pub enum FormattedTextInstruction {
    AddText(String),
    PushStyle(String),
    PopStyle(String),
    InsertPlaceholder(String),
}

pub fn parse(text: &str) -> Vec<FormattedTextInstruction> {
    let parser = parser::Parser::new(text);
    parser.parse()
}
