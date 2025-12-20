use crate::*;

mod format_options;
mod parser;
mod placeholder;
pub use format_options::*;
pub use parser::*;
pub use placeholder::*;

/// A parsed format string, consisting of literal parts and placeholders.
///
/// Structure:
///     "..........{some_type:some-config}..........{some_type:some-config}.........."
///      \________/\_____________________/\________/\_____________________/\________/
///       parts[0]     placeholders[0]     parts[1]     placeholders[1]     parts[2]
///                                  
pub struct FormatString<'a> {
    pub placeholders: Vec<Placeholder<'a>>,
    pub parts: Vec<String>, // contains placeholders.len() + 1 parts
}

impl<'a> FormatString<'a> {
    pub fn new(src: StrLitSlice<'a>, escape_input: bool) -> Result<Self> {
        let mut placeholders = vec![];
        let mut parts = vec![];
        let mut current_part = String::new();
        let mut current_part_start = 0;

        let mut parser = FormatStringParser::new(src);

        loop {
            let prev_pos = parser.get_pos();
            let Ok((pos, c)) = parser.take() else {
                break;
            };

            if c == '{' {
                if parser.take_if_eq('{').is_some() {
                    // escaped '{{', will be handled like a regular char by the following code
                } else {
                    let part = std::mem::take(&mut current_part);
                    if !escape_input && let Err(err) = regex_syntax::parse(&part) {
                        let msg = format!("invalid regex syntax in literal part: {err}");
                        return parser.slice(current_part_start, prev_pos).err(msg);
                    }
                    parts.push(part);
                    parser.mark_open_bracket(pos);
                    placeholders.push(parser.parse()?);
                    current_part_start = parser.get_pos();
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
                    return parser.err_at(pos, msg);
                } else {
                    // standalone '}'
                    let msg = "unexpected standalone '}'. Literal '}' need to be escaped as '}}'";
                    return parser.err_at(pos, msg);
                }
            }

            current_part.push(c);
        }

        if !escape_input && let Err(err) = regex_syntax::parse(&current_part) {
            let msg = format!("invalid regex syntax in literal part: {err}");
            return parser.slice(current_part_start, parser.get_pos()).err(msg);
        }
        parts.push(current_part);
        Ok(Self {
            placeholders,
            parts,
        })
    }
}

// TODO: add tests
