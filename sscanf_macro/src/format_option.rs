use crate::*;

pub const MISSING_CLOSE_STRING: &str = "missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'";

pub struct FormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub kind: FormatOptionKind,
}

pub enum FormatOptionKind {
    Radix { radix: u8, prefix: PrefixPolicy },
    Regex(String),
    Hashtag,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixPolicy {
    Forced,   // '#' + 'x', 'o', 'b'
    Optional, // just 'x', 'o', 'b'
    Never,    // custom radix 'r'
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
                Ok((Self { src, kind }, close_bracket_index))
            }
            '}' => {
                let msg = "format options cannot be empty. Consider removing the ':'";
                src.slice(start..=start).err(msg)
            }
            _ => Self::from_radix(input, src, start, outer_start),
        }
    }

    fn from_radix<I: Iterator<Item = (usize, char)>>(
        input: &mut std::iter::Peekable<I>,
        src: &StrLitSlice<'a>,
        start: usize,
        outer_start: usize,
    ) -> Result<(Self, usize)> {
        let (close_bracket_index, _) = input
            .find(|(_, c)| *c == '}')
            .ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?;

        let src = src.slice(start..close_bracket_index);

        let (radix, prefix) = match src.text() {
            "#" => {
                let kind = FormatOptionKind::Hashtag;
                return Ok((Self { src, kind }, close_bracket_index));
            }
            "x" => (16, PrefixPolicy::Optional),
            "o" => (8, PrefixPolicy::Optional),
            "b" => (2, PrefixPolicy::Optional),
            "#x" | "x#" => (16, PrefixPolicy::Forced),
            "#o" | "o#" => (8, PrefixPolicy::Forced),
            "#b" | "b#" => (2, PrefixPolicy::Forced),
            mut s => {
                let mut prefix = PrefixPolicy::Never;
                if let Some(inner) = s.strip_prefix('#').or_else(|| s.strip_suffix('#')) {
                    prefix = PrefixPolicy::Forced;
                    s = inner;
                }

                if let Some(n) = s.strip_prefix('r') {
                    let radix = n.parse::<u8>().map_err(|_| {
                        src.error("radix option 'r' has to be followed by a number")
                    })?;
                    if radix < 2 || radix > 36 {
                        // Range taken from: https://doc.rust-lang.org/std/primitive.usize.html#panics
                        return src.err("radix has to be a number between 2 and 36");
                    }
                    if prefix == PrefixPolicy::Forced && !matches!(radix, 2 | 8 | 16) {
                        return src.err("radix option '#' can only be used with base 2, 8 or 16");
                    }
                    (radix, prefix)
                } else {
                    let msg = "unrecognized format option.
Hint: Regex format options must start and end with '/'";
                    return src.err(msg);
                }
            }
        };

        let kind = FormatOptionKind::Radix { radix, prefix };
        Ok((Self { src, kind }, close_bracket_index))
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
