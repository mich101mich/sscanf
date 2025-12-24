use std::fmt::Display;

use crate::*;

pub const MISSING_CLOSE_STRING: &str = "missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'";

#[derive(Clone)]
pub struct FormatStringParser<'a> {
    src: StrLitSlice<'a>,
    /// the characters in the source string
    chars: Vec<char>,
    /// The byte indices of the characters in the source string. Same length as `chars`.
    char_indices: Vec<usize>,
    /// Index into `chars`/`char_indices` of the next character to take
    pos: usize,
    /// Index of the most recent open curly bracket
    open_bracket_pos: usize,
}

impl<'a> FormatStringParser<'a> {
    pub fn new(src: StrLitSlice<'a>) -> Self {
        let (char_indices, chars) = if src.is_raw() {
            src.text().char_indices().unzip()
        } else {
            unescape_regular_string(src.text())
        };
        Self {
            src,
            chars,
            char_indices,
            pos: 0,
            open_bracket_pos: usize::MAX, // invalid value, will be set on first mark
        }
    }

    pub fn get_pos(&self) -> usize {
        self.pos
    }

    pub fn mark_open_bracket(&mut self, pos: usize) {
        self.open_bracket_pos = pos;
    }
    pub fn get_open_bracket_pos(&self) -> usize {
        self.open_bracket_pos
    }

    /// Take the next character from the source string, if available.
    pub fn take(&mut self) -> Result<(usize, char)> {
        let ret = self.peek_required()?;
        self.pos += 1;
        Ok(ret)
    }
    pub fn take_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<(usize, char)> {
        let ret = self.peek()?;
        if !f(ret.1) {
            return None;
        }
        self.pos += 1;
        Some(ret)
    }
    pub fn take_if_eq(&mut self, c: char) -> Option<(usize, char)> {
        self.take_if(|x| x == c)
    }
    pub fn map_take_if<T>(&mut self, f: impl FnOnce(char) -> Option<T>) -> Option<(usize, T)> {
        let (pos, c) = self.peek()?;
        let ret = f(c)?;
        self.pos += 1;
        Some((pos, ret))
    }

    /// Return the next character without consuming it.
    pub fn peek(&mut self) -> Option<(usize, char)> {
        Some((self.pos, *self.chars.get(self.pos)?))
    }
    /// Return the next character without consuming it, returning an error if there is no next character.
    pub fn peek_required(&mut self) -> Result<(usize, char)> {
        match self.chars.get(self.pos) {
            Some(&c) => Ok((self.pos, c)),
            None => {
                if self.open_bracket_pos == usize::MAX {
                    // reached end without any placeholders, just return any error
                    Err(Error::new(Span::call_site(), ""))
                } else {
                    self.err_since(self.open_bracket_pos, MISSING_CLOSE_STRING)
                }
            }
        }
    }
    pub fn peek2(&mut self) -> Option<(usize, char)> {
        let next_pos = self.pos + 1;
        Some((next_pos, *self.chars.get(next_pos)?))
    }

    pub fn parse<T: FromFormatString<'a>>(&mut self) -> Result<T> {
        T::parse(self)
    }

    pub fn slice(&self, start: usize, end: usize) -> StrLitSlice<'a> {
        let start = self.char_indices[start];
        if let Some(end) = self.char_indices.get(end) {
            self.src.slice(start..*end)
        } else {
            self.src.slice(start..)
        }
    }
    pub fn slice_since(&self, start: usize) -> StrLitSlice<'a> {
        self.slice(start, self.pos)
    }

    pub fn err_since<T>(&self, start: usize, message: impl Display) -> Result<T> {
        self.slice_since(start).err(message)
    }
    pub fn err_at<T>(&self, pos: usize, message: impl Display) -> Result<T> {
        self.slice(pos, pos + 1).err(message)
    }
}

/// non-raw strings still contain escapes, which would be double escaped and misinterpreted if we kept them.
/// So we need to unescape them first.
fn unescape_regular_string(src: &str) -> (Vec<usize>, Vec<char>) {
    let mut char_indices = vec![];
    let mut chars = vec![];
    let mut iter = src.char_indices();
    const ERROR: &str =
        "sscanf: invalid escape sequence. This should have been caught by the Rust compiler";
    while let Some((i, c)) = iter.next() {
        if c != '\\' {
            char_indices.push(i);
            chars.push(c);
            continue;
        }
        let (_, next_c) = iter.next().expect(ERROR);
        // Source: <https://doc.rust-lang.org/reference/tokens.html#literals>
        // Regular strings can contain Quote, ASCII and Unicode escapes
        match next_c {
            // Quote escapes
            '"' => chars.push('"'),
            '\'' => chars.push('\''),

            // ASCII escapes
            'x' => {
                // hex escape: \xNN (always has exactly two hex digits)
                let mut hex = String::new();
                hex.push(iter.next().expect(ERROR).1);
                hex.push(iter.next().expect(ERROR).1);
                let byte = u8::from_str_radix(&hex, 16).expect(ERROR);
                chars.push(byte as char);
            }
            'n' => chars.push('\n'),
            'r' => chars.push('\r'),
            't' => chars.push('\t'),
            '\\' => chars.push('\\'),
            '0' => chars.push('\0'),

            // Unicode escapes
            'u' => {
                // Unicode escape: \u{NNNN...} (1-6 hex digits inside braces)
                let brace_open = iter.next().expect(ERROR).1;
                assert_eq!(brace_open, '{', "{ERROR}");
                let hex = iter
                    .by_ref()
                    .map(|(_, c)| c)
                    .take_while(|c| *c != '}')
                    .collect::<String>();
                let codepoint = u32::from_str_radix(&hex, 16).expect(ERROR);
                let character = std::char::from_u32(codepoint).expect(ERROR);
                chars.push(character);
            }

            _ => panic!(
                "sscanf: Unexpected escape sequence. If Rust has new syntax for string escapes, sscanf needs to be updated to match it, so please open an issue."
            ),
        }
        char_indices.push(i); // index of the '\'
    }

    (char_indices, chars)
}

/// Trait to parse a type using a `FormatStringParser`.
pub trait FromFormatString<'a>: Sized {
    fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        let src = str_lit! { "Hello1" };
        let mut parser = FormatStringParser::new(src.to_slice());

        // "Hello1"
        assert_eq!(parser.get_pos(), 0);
        assert_eq!(parser.peek(), Some((0, 'H')));
        assert_eq!(parser.take().ok(), Some((0, 'H')));
        assert_eq!(parser.get_pos(), 1);

        // "ello1"
        assert_eq!(parser.peek(), Some((1, 'e')));
        assert_eq!(parser.take_if_eq('e'), Some((1, 'e')));

        // "llo1"
        assert_eq!(parser.peek(), Some((2, 'l')));
        assert_eq!(parser.take_if(|c| c == 'l'), Some((2, 'l')));

        // "lo1"
        assert_eq!(parser.peek(), Some((3, 'l')));
        assert_eq!(parser.take_if(|c| c == 'x'), None);
        assert_eq!(parser.peek(), Some((3, 'l')));
        assert_eq!(parser.take().ok(), Some((3, 'l')));

        // "o1"
        assert_eq!(parser.map_take_if(|c| c.to_digit(10)), None);
        assert_eq!(parser.peek2(), Some((5, '1')));
        assert_eq!(parser.peek_required().ok(), Some((4, 'o')));
        assert_eq!(parser.take().ok(), Some((4, 'o')));
        assert_eq!(parser.get_pos(), 5);

        // "1"
        assert_eq!(parser.slice_since(1).text(), "ello");
        assert_eq!(parser.map_take_if(|c| c.to_digit(10)), Some((5, 1)));

        // ""
        assert_eq!(parser.peek(), None);
        assert!(parser.peek_required().is_err());
    }

    #[test]
    fn multibyte_test() {
        let src = str_lit! { "y̆😛y̆" };
        let mut parser = FormatStringParser::new(src.to_slice());

        assert_eq!(parser.slice(0, 1).text(), "y");
        assert_eq!(parser.slice(1, 2).text(), "̆"); // notice the practically invisible modifier char on top of the quote
        assert_eq!(parser.slice(2, 3).text(), "😛");
        assert_eq!(parser.slice(0, 2).text(), "y̆");
        assert_eq!(parser.slice(0, 5).text(), "y̆😛y̆");
        // Note: this is actually terrible behavior. Visually (and usually also when typing), the 'y̆' is a single
        // character, but Rust counts it as two separate chars.
        // The rest of the code only checks for ascii characters like '{', '}', ':', etc., so emoji and other utf-8
        // multibyte characters are completely fine, but I can't rule out that there won't be a modifier for one
        // of the chars that I'm looking for that breaks this.
        // However, fixing this is also way too complicated for now.

        // "y̆😛y̆"
        assert_eq!(parser.get_pos(), 0);
        assert_eq!(parser.peek(), Some((0, 'y')));
        assert_eq!(parser.take().ok(), Some((0, 'y')));
        assert_eq!(parser.get_pos(), 1);

        // "̆😛y̆"
        assert_eq!(parser.slice_since(0).text(), "y");
        assert_eq!(parser.peek(), Some((1, '̆'))); // again, on top of the quote
        assert_eq!(parser.take().ok(), Some((1, '̆')));
        assert_eq!(parser.get_pos(), 2);

        // "😛y̆"
        assert_eq!(parser.slice_since(0).text(), "y̆");
        assert_eq!(parser.peek(), Some((2, '😛')));
        assert_eq!(parser.take().ok(), Some((2, '😛')));
        assert_eq!(parser.get_pos(), 3);

        // "y̆"
        assert_eq!(parser.peek(), Some((3, 'y')));
        assert_eq!(parser.take().ok(), Some((3, 'y')));
        assert_eq!(parser.get_pos(), 4);

        // "̆"
        assert_eq!(parser.peek(), Some((4, '̆')));
        assert_eq!(parser.take().ok(), Some((4, '̆')));
        assert_eq!(parser.get_pos(), 5);

        // ""
        assert_eq!(parser.peek(), None);
        assert!(parser.peek_required().is_err());
    }

    #[test]
    fn test_parse_trait() {
        struct Number(usize);
        impl FromFormatString<'_> for Number {
            fn parse(parser: &mut FormatStringParser) -> Result<Self> {
                let Some((_, first_digit)) = parser.map_take_if(|c| c.to_digit(10)) else {
                    return parser.err_at(
                        parser.get_pos(),
                        "expected a digit at the start of a number",
                    );
                };
                let mut num = first_digit as usize;
                while let Some((_, digit)) = parser.map_take_if(|c| c.to_digit(10)) {
                    num = num * 10 + digit as usize;
                }
                Ok(Number(num))
            }
        }

        let src = str_lit! { "123abc" };
        let mut parser = FormatStringParser::new(src.to_slice());

        let number: Number = parser.parse().unwrap();
        assert_eq!(number.0, 123);
        assert_eq!(parser.peek(), Some((3, 'a')));

        assert!(parser.parse::<Number>().is_err()); // no digits at the start
    }

    #[test]
    fn test_unescape() {
        let src = str_lit! { "\n \\n \\ \0 \x41 \u{0041} " };
        let (indices, chars) = unescape_regular_string(src.to_slice().text());
        let char_indices = indices.into_iter().zip(chars).collect::<Vec<_>>();
        assert_eq!(
            char_indices,
            vec![
                (0, '\n'),
                (2, ' '),
                (3, '\\'),
                (5, 'n'),
                (6, ' '),
                (7, '\\'),
                (9, ' '),
                (10, '\0'),
                (12, ' '),
                (13, 'A'),
                (17, ' '),
                (18, 'A'),
                (26, ' '),
            ]
        );
    }
}
