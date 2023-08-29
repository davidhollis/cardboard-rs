use super::FormattedTextInstruction;

const DEFAULT_TAG_BUFFER_SIZE: usize = 24;

pub(super) struct Parser<'a> {
    text: &'a str,
    state: ParserState,
    current_text_buffer: String,
    current_tag_buffer: String,
}

impl<'a> Parser<'a> {
    pub(super) fn new(text: &'a str) -> Parser<'a> {
        Parser {
            text,
            state: ParserState::ReadingText,
            current_text_buffer: String::new(),
            current_tag_buffer: String::with_capacity(DEFAULT_TAG_BUFFER_SIZE),
        }
    }

    pub(super) fn parse(mut self) -> Vec<FormattedTextInstruction> {
        let mut instructions = vec![];

        for ch in self.text.chars() {
            match self.state {
                ParserState::ReadingText => match ch {
                    '<' => self.state = ParserState::ReadingStyleTag,
                    ':' => self.state = ParserState::ReadingPlaceholderSigil,
                    _ => self.current_text_buffer.push(ch),
                },
                ParserState::ReadingStyleTag => match ch {
                    ('A' ..= 'Z') | ('a' ..= 'z') | ('0' ..= '9') | '.' | '_' | '-' => {
                        self.current_tag_buffer.push(ch);
                        self.state = ParserState::ReadingStyleOpenTag;
                    },
                    '/' => self.state = ParserState::ReadingStyleCloseTag,
                    _ => {
                        // Abandon parsing a tag. Push the '<' we've already
                        // read and the character we're looking at.
                        self.current_text_buffer.push('<');
                        self.current_text_buffer.push(ch);
                        self.state = ParserState::ReadingText;
                    },
                },
                ParserState::ReadingStyleOpenTag => match ch {
                    ('A' ..= 'Z') | ('a' ..= 'z') | ('0' ..= '9') | '.' | '_' | '-' => {
                        self.current_tag_buffer.push(ch)
                    },
                    '>' => {
                        // Finish reading the tag
                        if !self.current_text_buffer.is_empty() {
                            instructions.push(FormattedTextInstruction::AddText(self.current_text_buffer));
                            self.current_text_buffer = String::new();
                        }
                        instructions.push(FormattedTextInstruction::PushStyle(self.current_tag_buffer));
                        self.current_tag_buffer = String::with_capacity(DEFAULT_TAG_BUFFER_SIZE);
                        self.state = ParserState::ReadingText;
                    },
                    _ => {
                        // Abandon parsing a tag. Push the '<' we've already
                        // read, the "tag name" we've read so far, and the
                        // character we're looking at.
                        self.current_text_buffer.push('<');
                        self.current_text_buffer.push_str(&self.current_tag_buffer);
                        self.current_text_buffer.push(ch);
                        self.current_tag_buffer = String::with_capacity(DEFAULT_TAG_BUFFER_SIZE);
                        self.state = ParserState::ReadingText;
                    },
                },
                ParserState::ReadingStyleCloseTag => match ch {
                    ('A' ..= 'Z') | ('a' ..= 'z') | ('0' ..= '9') | '.' | '_' | '-' => {
                        self.current_tag_buffer.push(ch)
                    },
                    '>' => {
                        if self.current_tag_buffer.is_empty() {
                            // Empty "close tags" ("</>") aren't actually tags
                            self.current_text_buffer.push_str("</>");
                            self.state = ParserState::ReadingText;
                        } else {
                            // Finish reading the tag
                            if !self.current_text_buffer.is_empty() {
                                instructions.push(FormattedTextInstruction::AddText(self.current_text_buffer));
                                self.current_text_buffer = String::new();
                            }
                            instructions.push(FormattedTextInstruction::PopStyle(self.current_tag_buffer));
                            self.current_tag_buffer = String::with_capacity(DEFAULT_TAG_BUFFER_SIZE);
                            self.state = ParserState::ReadingText;
                        }
                    },
                    _ => {
                        // Abandon parsing a tag. Push the '</' we've already
                        // read, the "tag name" we've read so far, and the
                        // character we're looking at.
                        self.current_text_buffer.push_str("</");
                        self.current_text_buffer.push_str(&self.current_tag_buffer);
                        self.current_text_buffer.push(ch);
                        self.current_tag_buffer = String::with_capacity(DEFAULT_TAG_BUFFER_SIZE);
                        self.state = ParserState::ReadingText;
                    },
                },
                ParserState::ReadingPlaceholderSigil => match ch {
                    ('A' ..= 'Z') | ('a' ..= 'z') | ('0' ..= '9') | '/' | '.' | '_' | '-' => {
                        self.current_tag_buffer.push(ch)
                    },
                    ':' => {
                        if self.current_tag_buffer.is_empty() {
                            // Empty "placeholders" ("::") aren't actually placeholders
                            self.current_text_buffer.push_str("::");
                            self.state = ParserState::ReadingText;
                        } else {
                            // Finish reading the placeholder
                            if !self.current_text_buffer.is_empty() {
                                instructions.push(FormattedTextInstruction::AddText(self.current_text_buffer));
                                self.current_text_buffer = String::new();
                            }
                            instructions.push(FormattedTextInstruction::InsertPlaceholder(self.current_tag_buffer));
                            self.current_tag_buffer = String::with_capacity(DEFAULT_TAG_BUFFER_SIZE);
                            self.state = ParserState::ReadingText;
                        }
                    },
                    _ => {
                        // Abandon parsing a placeholder. Push the ':' we've
                        // already read, the "placeholder name" we've read so
                        // far, and the character we're looking at.
                        self.current_text_buffer.push(':');
                        self.current_text_buffer.push_str(&self.current_tag_buffer);
                        self.current_text_buffer.push(ch);
                        self.current_tag_buffer = String::with_capacity(DEFAULT_TAG_BUFFER_SIZE);
                        self.state = ParserState::ReadingText;
                    },
                },
            }
        }

        if !self.current_text_buffer.is_empty() {
            instructions.push(FormattedTextInstruction::AddText(self.current_text_buffer));
        }

        instructions
    }
}

enum ParserState {
    ReadingText,
    ReadingStyleTag,
    ReadingStyleOpenTag,
    ReadingStyleCloseTag,
    ReadingPlaceholderSigil,
}

#[cfg(test)]
mod tests {
    use crate::format::FormattedTextInstruction;

    use super::Parser;

    #[test]
    fn it_finds_well_formed_elements() -> () {
        let text = "Some text with <b>nested <i>tags</i></b> and even an :emoji: for good measure";
        let parser = Parser::new(text);
        let formatting_instructions = parser.parse();

        assert_eq!(
            formatting_instructions,
            vec![
                FormattedTextInstruction::AddText("Some text with ".to_string()),
                FormattedTextInstruction::PushStyle("b".to_string()),
                FormattedTextInstruction::AddText("nested ".to_string()),
                FormattedTextInstruction::PushStyle("i".to_string()),
                FormattedTextInstruction::AddText("tags".to_string()),
                FormattedTextInstruction::PopStyle("i".to_string()),
                FormattedTextInstruction::PopStyle("b".to_string()),
                FormattedTextInstruction::AddText(" and even an ".to_string()),
                FormattedTextInstruction::InsertPlaceholder("emoji".to_string()),
                FormattedTextInstruction::AddText(" for good measure".to_string()),
            ]
        )
    }

    #[test]
    fn it_degrades_invalid_forms_to_text() -> () {
        let text = r###"
            This should just be one big text instruction:
              - open tag <with spaces>
              - close tag </with spaces>
              - emoji :with spaces:
              - empty open tag <>,
              - empty close tag </>
              - empty emoji ::
        "###;
        let parser = Parser::new(text);
        let instructions = parser.parse();

        assert_eq!(
            instructions,
            vec![
                FormattedTextInstruction::AddText(text.to_string()),
            ]
        )
    }
}
