use super::*;

use unicode_segmentation::UnicodeSegmentation;

pub struct FormatString<'a> {
    pub src: StrLitSlice<'a>,
    pub placeholders: Vec<Placeholder<'a>>,
    pub parts: Vec<String>, // contains placeholders.len() + 1 escaped parts
}

pub enum GraphemeItem<'a> {
    Char(char),
    Other(&'a str),
}
impl GraphemeItem<'_> {
    pub fn as_char(&self) -> Option<char> {
        match self {
            GraphemeItem::Char(c) => Some(*c),
            GraphemeItem::Other(_) => None,
        }
    }
    pub fn push_to(&self, out: &mut String) {
        match self {
            GraphemeItem::Char(c) => out.push(*c),
            GraphemeItem::Other(s) => out.push_str(s),
        }
    }
}
impl PartialEq<char> for GraphemeItem<'_> {
    fn eq(&self, other: &char) -> bool {
        match self {
            GraphemeItem::Char(c) => c == other,
            GraphemeItem::Other(_) => false,
        }
    }
}

impl<'a> FormatString<'a> {
    pub fn new(src: StrLitSlice<'a>, escape_input: bool) -> Result<Self> {
        let mut placeholders = vec![];
        let mut parts = vec![];
        let mut current_part = String::new();

        // keep the iterator as a variable to allow peeking and advancing in a sub-function
        let mut iter = src
            .text
            .graphemes(true)
            .map(|s| {
                let mut iter = s.chars();
                let c = iter.next().unwrap();
                if iter.next().is_none() {
                    GraphemeItem::Char(c)
                } else {
                    GraphemeItem::Other(s)
                }
            })
            .enumerate()
            .peekable();

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

            if escape_input
                && c.as_char()
                    .map(regex_syntax::is_meta_character)
                    .unwrap_or(false)
            {
                current_part.push('\\');
            }

            c.push_to(&mut current_part);
        }

        parts.push(current_part);
        Ok(Self {
            src,
            placeholders,
            parts,
        })
    }
}
