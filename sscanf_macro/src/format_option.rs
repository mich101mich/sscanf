use crate::*;

pub const MISSING_CLOSE_STRING: &str = "missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'";

pub struct FormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub kind: FormatOptionKind,
}

pub enum FormatOptionKind {
    Radix(u8),
    Regex(String),
}

impl<'a> FormatOption<'a> {
    pub fn new<'b, I: Iterator<Item = (usize, GraphemeItem<'b>)>>(
        input: &'_ mut std::iter::Peekable<I>,
        src: &'_ StrLitSlice<'a>,
        outer_start: usize,
    ) -> Result<(Self, usize)> {
        let (start, c) = input
            .next()
            .ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?;

        match c {
            GraphemeItem::Char('x' | 'o' | 'b' | 'r') => {
                let (radix, slice, end) =
                    FormatOption::get_radix(input, src, c.as_char().unwrap(), start, outer_start)?;
                let ret = Self {
                    src: slice,
                    kind: FormatOptionKind::Radix(radix),
                };
                Ok((ret, end))
            }
            GraphemeItem::Char('/') => {
                let mut end = None;
                let mut regex = String::new();
                while let Some((i, c)) = input.next() {
                    if c == '/' {
                        end = Some(i);
                        break;
                    } else if c == '\\' {
                        let (_, next) = input
                            .next()
                            .ok_or_else(|| src.slice(i..).error("unexpected end of regex"))?;
                        if next != '/' {
                            regex.push('\\');
                        }
                        next.push_to(&mut regex);
                    } else {
                        c.push_to(&mut regex);
                    }
                }

                let end =
                    end.ok_or_else(|| src.slice(start + 1..).error("missing '/' to end regex"))?;

                // take } from input
                if !input.next().map(|(_, c)| c == '}').unwrap_or(false) {
                    let msg = "closing '/' has to be followed by '}'";
                    return src.slice(end..=end + 1).err(msg);
                }

                let src = src.slice(start..=end);
                if let Err(err) = regex_syntax::Parser::new().parse(&regex) {
                    let msg = format!("{}\n\nIn custom Regex format option", err);
                    return src.err(&msg);
                }
                let kind = FormatOptionKind::Regex(regex);
                return Ok((Self { src, kind }, end + 1));
            }
            GraphemeItem::Char('}') => {
                let msg = "format options cannot be empty. Consider removing the ':'";
                return src.slice(start..=start + 1).err(msg);
            }
            _ => {
                let msg = "unrecognized format option.
Hint: Regex format options must start and end with '/'";
                return src.slice(start..=start).err(msg);
            }
        }
    }

    fn get_radix<'b, I: Iterator<Item = (usize, GraphemeItem<'b>)>>(
        input: &'_ mut std::iter::Peekable<I>,
        src: &'_ StrLitSlice<'a>,
        c: char,
        start: usize,
        outer_start: usize,
    ) -> Result<(u8, StrLitSlice<'a>, usize)> {
        let mut end = None;
        while let Some((i, c)) = input.next() {
            if c == '}' {
                end = Some(i);
                break;
            } else if !c.as_char().map(|c| c.is_numeric()).unwrap_or(false) {
                return src.slice(i..=i).err(
                    "invalid character in radix option.
Hint: Regex format options must start and end with '/'",
                );
            }
        }
        let end = end.ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?;
        let slice = src.slice(start..end);

        let radix: u8 = if c == 'r' {
            src.slice(start + 1..end).text.parse().map_err(|_| {
                slice.error(
                    "invalid radix option.
Hint: Regex format options must start and end with '/'",
                )
            })?
        } else {
            if end != start + 1 {
                return slice.err(
                    "unrecognized radix option.
Hint: Regex format options must start and end with '/'",
                );
            }
            match c {
                'x' => 16,
                'o' => 8,
                'b' => 2,
                _ => unreachable!(),
            }
        };
        if radix < 2 || radix > 36 {
            // Range taken from: https://doc.rust-lang.org/std/primitive.usize.html#panics
            return slice.err("radix must be between 2 and 36");
        }
        Ok((radix, slice, end))
    }
}
