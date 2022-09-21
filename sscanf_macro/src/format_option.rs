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
    pub fn new<I: Iterator<Item = (usize, char)>>(
        input: &'_ mut std::iter::Peekable<I>,
        src: &'_ StrLitSlice<'a>,
        outer_start: usize,
    ) -> Result<(Self, usize)> {
        let (start, c) = input
            .next()
            .ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?;

        match c {
            'x' | 'o' | 'b' | 'r' => {
                let (radix, slice, end) =
                    FormatOption::get_radix(input, src, c, start, outer_start)?;
                let ret = Self {
                    src: slice,
                    kind: FormatOptionKind::Radix(radix),
                };
                Ok((ret, end))
            }
            '/' => {
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
                        regex.push(next);
                    } else {
                        regex.push(c);
                    }
                }

                let end =
                    end.ok_or_else(|| src.slice(start..).error("missing '/' to end regex"))?;

                // take } from input
                let close_bracket_index = if let Some((i, c)) = input.next() {
                    if c != '}' {
                        return src
                            .slice(i..=i)
                            .err("end of regex '/' has to be followed by end of placeholder '}'");
                    }
                    i
                } else {
                    return src.slice(outer_start..).err(MISSING_CLOSE_STRING);
                };

                let src = src.slice(start..=end);
                match regex_syntax::Parser::new().parse(&regex) {
                    Ok(hir) => {
                        if contains_capture_group(&hir) {
                            let msg = "custom regex can't contain capture groups.
Either make them non-capturing by adding '?:' after the '(' or remove/escape the '(' and ')'";
                            return src.err(msg);
                        }
                    }
                    Err(err) => {
                        let msg = format!("{}\n\nIn custom Regex format option", err);
                        return src.err(&msg);
                    }
                }
                let kind = FormatOptionKind::Regex(regex);
                return Ok((Self { src, kind }, close_bracket_index));
            }
            '}' => {
                let msg = "format options cannot be empty. Consider removing the ':'";
                return src.slice(start..=start).err(msg);
            }
            _ => {
                let msg = "unrecognized format option.
Hint: Regex format options must start and end with '/'";
                return src.slice(start..=start).err(msg);
            }
        }
    }

    fn get_radix<I: Iterator<Item = (usize, char)>>(
        input: &'_ mut std::iter::Peekable<I>,
        src: &'_ StrLitSlice<'a>,
        c: char,
        start: usize,
        outer_start: usize,
    ) -> Result<(u8, StrLitSlice<'a>, usize)> {
        let mut number_offset = None;
        let mut end = None;

        while let Some((i, c)) = input.next() {
            if c == '}' {
                end = Some(i);
                break;
            } else if number_offset.is_none() {
                number_offset = Some(i - start);
            }
        }

        let end = end.ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?;
        let slice = src.slice(start..end);

        let radix: u8 = if c == 'r' {
            let number_offset = number_offset.ok_or_else(|| {
                slice.error("radix option 'r' must be followed by the radix number")
            })?;

            let number = slice.slice(number_offset..);

            number.text().parse().map_err(|_| {
                let msg = "invalid number after radix option 'r'.
Hint: If this was meant to be a regex option, surround it with '/'";
                number.error(msg)
            })?
        } else {
            if let Some(number_offset) = number_offset {
                let msg = "radix options 'x', 'o', 'b' cannot be followed by anything.
Hint: If this was meant to be a regex option, surround it with '/'";
                return slice.slice(number_offset..).err(msg);
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

fn contains_capture_group(hir: &regex_syntax::hir::Hir) -> bool {
    use regex_syntax::hir::HirKind::*;
    match hir.kind() {
        Group(g) => {
            if g.kind != regex_syntax::hir::GroupKind::NonCapturing {
                return true;
            }
            contains_capture_group(g.hir.as_ref())
        }
        Concat(c) | Alternation(c) => c.iter().any(contains_capture_group),
        Repetition(r) => contains_capture_group(r.hir.as_ref()),
        _ => false,
    }
}
