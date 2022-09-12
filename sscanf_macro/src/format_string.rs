use super::*;

pub struct FormatString<'a> {
    pub src: StrLitSlice<'a>,
    pub placeholders: Vec<Placeholder<'a>>,
    pub parts: Vec<String>, // contains placeholders.len() + 1 escaped parts
}

impl<'a> FormatString<'a> {
    pub fn new(src: StrLitSlice<'a>, escape_input: bool) -> Result<Self> {
        let mut placeholders = vec![];
        let mut parts = vec![];
        let mut current_part = String::new();

        // keep the iterator as a variable to allow peeking and advancing in a sub-function
        let mut iter = src.text().char_indices().peekable();

        while let Some((i, c)) = iter.next() {
            if c == '{' {
                if iter.next_if(|(_, c)| *c == '{').is_some() {
                    // escaped '{{', will be handled like a regular char by the following code
                } else {
                    placeholders.push(Placeholder::new(&mut iter, &src, i)?);
                    current_part.push('(');
                    parts.push(current_part);
                    current_part = String::from(")");
                    continue;
                }
            } else if c == '}' {
                if iter.next_if(|(_, c)| *c == '}').is_some() {
                    // escaped '}}', will be handled like a regular char by the following code
                } else {
                    return src
                        .slice(i..=i)
                        .err("Unexpected standalone '}'. Literal '}' need to be escaped as '}}'");
                }
            }

            if escape_input && regex_syntax::is_meta_character(c) {
                current_part.push('\\');
            }

            current_part.push(c);
        }

        parts.push(current_part);
        Ok(Self {
            src,
            placeholders,
            parts,
        })
    }
}
