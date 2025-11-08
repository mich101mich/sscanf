use crate::*;

mod format_options;
mod parser;
mod placeholder;
pub use format_options::*;
pub use parser::*;
pub use placeholder::*;

pub struct FormatString<'a> {
    pub placeholders: Vec<Placeholder<'a>>,
    pub parts: Vec<String>, // contains placeholders.len() + 1 parts
}

impl<'a> FormatString<'a> {
    pub fn new(src: StrLitSlice<'a>) -> Result<Self> {
        let mut placeholders = vec![];
        let mut parts = vec![];
        let mut current_part = String::new();

        let mut parser = FormatStringParser::new(src);

        while let Ok((pos, c)) = parser.take() {
            if c == '{' {
                if parser.take_if_eq('{').is_some() {
                    // escaped '{{', will be handled like a regular char by the following code
                } else {
                    parts.push(std::mem::take(&mut current_part));
                    parser.mark_open_bracket(pos);
                    placeholders.push(Placeholder::parse(&mut parser)?);
                    continue;
                }
            } else if c == '}' {
                if parser.take_if_eq('}').is_some() {
                    // escaped '}}', will be handled like a regular char by the following code
                } else if current_part.is_empty() && !placeholders.is_empty() {
                    // most recent chars were a placeholder: '{...}}'
                    let msg = "escaped '}}' after an unescaped '{'.
If you didn't mean to create a placeholder, escape the '{' as '{{'
If you did, either remove the second '}' or escape it with another '}'";
                    return parser.err_at(pos, msg); // checked in tests/fail/<channel>/invalid_placeholder.rs
                } else {
                    // standalone '}'
                    let msg = "unexpected standalone '}'. Literal '}' need to be escaped as '}}'";
                    return parser.err_at(pos, msg); // checked in tests/fail/<channel>/missing_bracket.rs
                }
            }

            current_part.push(c);
        }

        parts.push(current_part);
        Ok(Self {
            placeholders,
            parts,
        })
    }
}

// TODO: add tests
