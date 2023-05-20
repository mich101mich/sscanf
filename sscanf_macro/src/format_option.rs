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
    Forced(PrefixKind),   // '#' + 'x', 'o', 'b'
    Optional(PrefixKind), // just 'x', 'o', 'b'
    Never,                // custom radix 'r'
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefixKind {
    Hex,
    Octal,
    Binary,
}
impl std::fmt::Display for PrefixKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefixKind::Hex => write!(f, "0x"),
            PrefixKind::Octal => write!(f, "0o"),
            PrefixKind::Binary => write!(f, "0b"),
        }
    }
}
impl ToTokens for PrefixKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PrefixKind::Hex => tokens.extend(quote! { Hex }),
            PrefixKind::Octal => tokens.extend(quote! { Octal }),
            PrefixKind::Binary => tokens.extend(quote! { Binary }),
        }
    }
}

impl<'a> FormatOption<'a> {
    pub fn new<I: Iterator<Item = (usize, char)>>(
        input: &'_ mut std::iter::Peekable<I>,
        src: &'_ StrLitSlice<'a>,
        outer_start: usize,
    ) -> Result<(Self, usize)> {
        let (start, c) = input
            .next()
            .ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?; // checked in tests/fail/<channel>/invalid_placeholder.rs

        match c {
            '/' => {
                let mut end = None;
                let mut regex = String::new();
                let mut escape = None;
                while let Some((i, c)) = input.next() {
                    if c == '/' {
                        if escape.take().is_some() {
                            regex.push('/');
                        } else {
                            end = Some(i);
                            break;
                        }
                    } else if c == '\\' {
                        if !src.is_raw() {
                            let (_, next) = input
                                .next()
                                .ok_or_else(|| src.slice(i..).error("unexpected end of regex"))?;
                            // the above error is probably not possible, since a single \ at
                            // the end of a non-raw string would escape the closing " and the
                            // compiler would already complain about that.
                            // the check is still here just in case

                            if next != '\\' {
                                // regular escaped char (\n, \t, etc)
                                if escape.take().is_some() {
                                    regex.push('\\');
                                }
                                regex.push('\\');
                                regex.push(next);
                                continue;
                            }
                        }
                        if escape.take().is_some() {
                            regex.push('\\');
                            regex.push('\\');
                        } else {
                            escape = Some(i);
                        }
                    } else {
                        if escape.take().is_some() {
                            regex.push('\\');
                        }
                        regex.push(c);
                    }
                }
                if let Some(i) = escape {
                    return src.slice(i..).err("unexpected end of regex"); // checked in tests/fail/<channel>/invalid_custom_regex.rs
                }
                let end =
                    end.ok_or_else(|| src.slice(start..).error("missing '/' to end regex"))?; // checked in tests/fail/<channel>/invalid_custom_regex.rs

                // take } from input
                let close_bracket_index = if let Some((i, c)) = input.next() {
                    if c != '}' {
                        let msg = "end of regex '/' has to be followed by end of placeholder '}'";
                        return src.slice(i..=i).err(msg); // checked in tests/fail/<channel>/invalid_custom_regex.rs
                    }
                    i
                } else {
                    return src.slice(outer_start..).err(MISSING_CLOSE_STRING); // checked in tests/fail/<channel>/invalid_placeholder.rs
                };

                let src = src.slice(start..=end);

                match regex_syntax::Parser::new().parse(&regex) {
                    Ok(hir) => {
                        if contains_capture_group(&hir) {
                            let msg = "custom regex cannot contain capture groups '(...)'.
Either make them non-capturing by adding '?:' after the '(' or remove/escape the '(' and ')'";
                            return src.err(msg);
                        }
                    }
                    Err(err) => {
                        let msg = format!("{}\n\nIn custom Regex format option", err);
                        return src.err(&msg); // checked in tests/fail/<channel>/invalid_custom_regex.rs
                    }
                }

                let kind = FormatOptionKind::Regex(regex);
                Ok((Self { src, kind }, close_bracket_index))
            }
            '}' => {
                let msg = "format options cannot be empty. Consider removing the ':'";
                src.slice(start..=start).err(msg) // checked in tests/fail/<channel>/invalid_placeholder.rs
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
            .ok_or_else(|| src.slice(outer_start..).error(MISSING_CLOSE_STRING))?; // checked in tests/fail/<channel>/invalid_placeholder.rs

        let src = src.slice(start..close_bracket_index);

        let (radix, prefix) = match src.text() {
            "#" => {
                let kind = FormatOptionKind::Hashtag;
                return Ok((Self { src, kind }, close_bracket_index));
            }
            "x" => (16, PrefixPolicy::Optional(PrefixKind::Hex)),
            "o" => (8, PrefixPolicy::Optional(PrefixKind::Octal)),
            "b" => (2, PrefixPolicy::Optional(PrefixKind::Binary)),
            "#x" | "x#" => (16, PrefixPolicy::Forced(PrefixKind::Hex)),
            "#o" | "o#" => (8, PrefixPolicy::Forced(PrefixKind::Octal)),
            "#b" | "b#" => (2, PrefixPolicy::Forced(PrefixKind::Binary)),
            s => {
                if s.starts_with('#') || s.ends_with('#') {
                    let msg = "config modifier '#' can only be used with 'x', 'o' or 'b'";
                    return src.err(msg); // checked in tests/fail/<channel>/invalid_radix_option.rs
                }

                if let Some(n) = s.strip_prefix('r') {
                    let radix = n.parse::<u8>().map_err(|_| {
                        let msg = "radix option 'r' has to be followed by a number";
                        src.error(msg) // checked in tests/fail/<channel>/invalid_radix_option.rs
                    })?;
                    if !(2..=36).contains(&radix) {
                        // Range taken from: https://doc.rust-lang.org/std/primitive.usize.html#panics
                        let msg = "radix has to be a number between 2 and 36";
                        return src.err(msg); // checked in tests/fail/<channel>/invalid_radix_option.rs
                    }
                    (radix, PrefixPolicy::Never)
                } else {
                    let msg = "unrecognized format option.
Hint: Regex format options must start and end with '/'";
                    return src.err(msg); // checked in tests/fail/<channel>/raw_string.rs
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
        // regex-syntax 0.7+ version
        // Capture(_) => true,
        // Concat(c) | Alternation(c) => c.iter().any(contains_capture_group),
        // Repetition(r) => contains_capture_group(r.sub.as_ref()),
        // _ => false,
    }
}
