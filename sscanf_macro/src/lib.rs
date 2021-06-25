//! Crate with proc_macros for sscanf. Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input,
    spanned::Spanned,
    Expr, LitStr, Token, TypePath,
};

struct PlaceHolder {
    name: String,
    config: Option<String>,
    span: (usize, usize),
}

struct SscanfInner {
    fmt: String,
    fmt_span: Literal,
    span_offset: usize,
    type_tokens: Vec<TypePath>,
}
struct Sscanf {
    src_str: Expr,
    inner: SscanfInner,
}

impl Parse for SscanfInner {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(Span::call_site(), "Missing format string"));
        }

        let fmt: LitStr = input.parse()?;
        let span_offset = {
            // this is a dirty hack to see if the literal is a raw string, which is necessary for
            // subspan to skip the 'r', 'r#', ...
            // this should be a thing that I can check on LitStr or Literal or whatever, but nooo,
            // I have to print it into a String via a TokenStream and check that one -.-
            //
            // "fun" fact: This used to actually be a thing in syn 0.11 where `Lit::Str` gave _two_
            // values: the literal and a `StrStyle`. This was apparently removed at some point for
            // being TOO USEFUL.
            let lit = fmt.to_token_stream().to_string();
            lit.chars().enumerate().find(|c| c.1 == '"').unwrap().0
        };

        // subspan only exists on Literal, but in order to get the content of the literal we need
        // LitStr, because once again convenience is a luxury
        let mut fmt_span = Literal::string(&fmt.value());
        fmt_span.set_span(fmt.span()); // fmt is a single Token so span() works even on stable

        let type_tokens;
        if input.is_empty() {
            type_tokens = vec![];
        } else {
            input.parse::<Token![,]>()?;

            type_tokens = input
                .parse_terminated::<_, Token![,]>(TypePath::parse)?
                .into_iter()
                .collect();
        }

        Ok(SscanfInner {
            fmt: fmt.value(),
            fmt_span,
            span_offset,
            type_tokens,
        })
    }
}
impl Parse for Sscanf {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "At least 2 Parameters required: Input and format string",
            ));
        }
        let src_str = input.parse()?;
        if input.is_empty() {
            return Err(Error::new(
                Span::call_site(),
                "At least 2 Parameters required: Missing format string",
            ));
        }
        let comma = input.parse::<Token![,]>()?;
        if input.is_empty() {
            return Err(Error::new_spanned(
                comma,
                "At least 2 Parameters required: Missing format string",
            ));
        }
        let inner = input.parse()?;

        Ok(Sscanf { src_str, inner })
    }
}

#[proc_macro]
pub fn scanf(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Sscanf);
    scanf_internal(input, true)
}

#[proc_macro]
pub fn scanf_unescaped(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Sscanf);
    scanf_internal(input, false)
}

#[proc_macro]
pub fn scanf_get_regex(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as SscanfInner);
    let (regex, _) = match generate_regex(input, true) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    quote!({
        #regex
        REGEX.clone()
    })
    .into()
}

fn scanf_internal(input: Sscanf, escape_input: bool) -> TokenStream1 {
    let (regex, matcher) = match generate_regex(input.inner, escape_input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let src_str = {
        let src_str = input.src_str;
        let (start, end) = full_span(&src_str);
        let mut param = quote_spanned!(start => &);
        param.extend(quote_spanned!(end => (#src_str)));
        quote!(::std::convert::AsRef::<str>::as_ref(#param))
    };
    quote!(
        {
            #regex
            #[allow(clippy::needless_question_mark)]
            REGEX.captures(#src_str).and_then(|cap| Some(( #(#matcher),* )))
        }
    )
    .into()
}

fn generate_regex(
    input: SscanfInner,
    escape_input: bool,
) -> Result<(TokenStream, Vec<TokenStream>)> {
    let (placeholders, regex_parts) = parse_format_string(&input, escape_input)?;

    // generate error with excess parts if lengths do not match
    let mut error = TokenStream::new();
    for ph in placeholders.iter().skip(input.type_tokens.len()) {
        let message = format!(
            "Missing Type for given '{{{}}}' Placeholder",
            ph.config.as_deref().unwrap_or("")
        );
        error.extend(sub_error(&message, &input, ph.span).to_compile_error());
    }
    for ty in input.type_tokens.iter().skip(placeholders.len()) {
        error.extend(
            Error::new_spanned(ty, "More Types than '{}' Placeholders provided").to_compile_error(),
        );
    }
    if !error.is_empty() {
        error.extend(quote!(let REGEX = ::sscanf::regex::Regex::new("").unwrap();));
        return Ok((error, vec![]));
    }

    // these need to be Vec instead of direct streams to allow comma separators
    let mut regex_builder = vec![];
    let mut match_grabber = vec![];
    for ((ph, ty), regex_prefix) in placeholders
        .iter()
        .zip(input.type_tokens.iter())
        .zip(regex_parts.iter())
    {
        regex_builder.push(quote!(#regex_prefix));
        if let Some(config) = ph.config.as_ref() {
            let (regex, matcher) = regex_from_config(config, ty, &ph, &input)?;
            regex_builder.push(regex);
            match_grabber.push(matcher);
        } else {
            let (start, end) = full_span(&ty);
            let name = &ph.name;

            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::sscanf::RegexRepresentation>::REGEX));
            regex_builder.push(s);

            s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::std::str::FromStr>::from_str(cap.name(#name)?.as_str()).ok()?));
            match_grabber.push(s);
        }
    }

    let last_regex = &regex_parts[placeholders.len()];
    regex_builder.push(quote!(#last_regex));

    let regex = quote!(::sscanf::lazy_static::lazy_static! {
        static ref REGEX: ::sscanf::regex::Regex = ::sscanf::regex::Regex::new(
            ::sscanf::const_format::concatcp!( #(#regex_builder),* )
        ).expect("sscanf cannot generate Regex");
    });

    Ok((regex, match_grabber))
}

fn parse_format_string(
    input: &SscanfInner,
    escape_input: bool,
) -> Result<(Vec<PlaceHolder>, Vec<String>)> {
    let mut placeholders = vec![];

    let mut regex = vec![];
    let mut current_regex = String::from("^");

    let mut name_index = 1;

    // iter as var to allow peeking and advancing in sub-function
    let mut iter = input.fmt.chars().enumerate().peekable();

    while let Some((i, c)) = iter.next() {
        if c == '{' {
            if let Some(mut ph) = parse_bracket_content(&mut iter, &input, i)? {
                ph.name = format!("type_{}", name_index);
                name_index += 1;

                current_regex += &format!("(?P<{}>", ph.name);
                regex.push(current_regex);
                current_regex = String::from(")");

                placeholders.push(ph);
                continue;
            }
            // else => escaped '{{', handle like regular char
        } else if c == '}' {
            // next_if_eq success => escaped '}}', iterator advanced, handle like regular char
            iter.next_if_eq(&(i + 1, '}')).ok_or_else(|| {
                sub_error(
                    "Unexpected standalone '}'. Literal '}' need to be escaped as '}}'",
                    &input,
                    (i, i),
                )
            })?;
        }

        if escape_input && regex_syntax::is_meta_character(c) {
            current_regex.push('\\');
        }

        current_regex.push(c);
    }

    current_regex.push('$');
    regex.push(current_regex);

    Ok((placeholders, regex))
}

fn parse_bracket_content<I: Iterator<Item = (usize, char)>>(
    input: &mut std::iter::Peekable<I>,
    src: &SscanfInner,
    start: usize,
) -> Result<Option<PlaceHolder>> {
    let mut config = String::new();
    while let Some((i, c)) = input.next() {
        if c == '\\' {
            // escape any curly brackets
            if let Some((_, c)) = input.next_if(|x| x.1 == '{' || x.1 == '}') {
                config.push(c);
                continue;
            }
        } else if c == '{' {
            if i == start + 1 {
                // '{' followed by '{' => escaped '{{'
                return Ok(None);
            }
            return sub_error_result(
                "Expected '}' after given '{', got '{' instead. Curly Brackets inside format options must be escaped with '\\'",
                src,
                (start, i),
            );
        } else if c == '}' {
            return Ok(Some(PlaceHolder {
                name: String::new(),
                config: if i == start + 1 { None } else { Some(config) },
                span: (start, i),
            }));
        } else {
            // anything else is part of the config
            config.push(c);
        }
    }
    // end of input string
    sub_error_result(
        "Missing '}' after given '{'. Literal '{' need to be escaped as '{{'",
        src,
        (start, src.fmt.len()),
    )
}

fn regex_from_config(
    config: &str,
    ty: &TypePath,
    ph: &PlaceHolder,
    src: &SscanfInner,
) -> Result<(TokenStream, TokenStream)> {
    let ty_string = ty.to_token_stream().to_string();
    if let Some(radix) = get_radix(config) {
        let binary_digits = binary_length(&ty_string).ok_or_else(|| {
            Error::new_spanned(
                ty,
                "radix options only work on primitive numbers from std with no path or type alias",
            )
        })?;
        // digit conversion: digits_base / log_x(base) * log_x(target) with any log-base x,
        // so we choose log_2 where log_2(target) = 1;
        let binaries_per_digit = (radix as f32).log2();
        let digits = (binary_digits as f32 / binaries_per_digit).ceil() as u8;

        // possible characters for digits
        use std::cmp::Ordering::*;
        let mut regex = match radix.cmp(&10) {
            Less => format!(r"[0-{}]", radix - 1),
            Equal => r"[0-9aA]".to_string(),
            Greater => {
                let last_letter = (b'a' + radix - 10) as char;
                let last_letter_upper = (b'A' + radix - 10) as char;
                format!(r"[0-9a-{}A-{}]", last_letter, last_letter_upper)
            }
        };
        // repetition factor
        regex += &format!(r"{{1, {}}}", digits);

        // optional prefix
        let prefix = if radix == 16 {
            Some("0x")
        } else if radix == 8 {
            Some("0o")
        } else if radix == 2 {
            Some("0b")
        } else {
            None
        };
        if let Some(pref) = prefix.as_ref() {
            regex = format!(r"{}{1}|{1}", pref, regex);
        }

        let span = ty.span();
        let name = &ph.name;
        let radix = radix as u32;
        let matcher = if let Some(prefix) = prefix {
            quote_spanned!(span => #ty::from_str_radix({
                let s = cap.name(#name)?.as_str();
                s.strip_prefix(#prefix).unwrap_or(s)
            }, #radix).ok()?)
        } else {
            quote_spanned!(span => #ty::from_str_radix(cap.name(#name)?.as_str(), #radix).ok()?)
        };

        Ok((quote!(#regex), matcher))
    } else {
        match ty_string.as_str() {
            "DateTime" | "NaiveDate" | "NaiveTime" | "NaiveDateTime" => {
                let (regex, chrono_fmt) = map_chrono_format(config, src, ph.span.0)?;

                let span = ty.span();
                let name = &ph.name;
                let matcher = quote_spanned!(span =>
                    ::sscanf::chrono::#ty::parse_from_str(cap.name(#name)?.as_str(), #chrono_fmt)
                        .expect("chrono parsing should be exact")
                );

                Ok((quote!(#regex), matcher))
            }
            "Utc" | "Local" => {
                let (regex, chrono_fmt) = map_chrono_format(config, src, ph.span.0)?;

                let span = ty.span();
                let name = &ph.name;
                let matcher = quote_spanned!(span =>
                    #ty.datetime_from_str(cap.name(#name)?.as_str(), #chrono_fmt).ok()?
                );

                Ok((quote!(#regex), matcher))
            }
            _ => sub_error_result(
                &format!("Unknown format option: '{}'", config),
                src,
                ph.span,
            ),
        }
    }
}

fn get_radix(config: &str) -> Option<u8> {
    if let Some(n) = config
        .strip_prefix('r')
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| *n >= 2 && *n <= 36)
    {
        return Some(n);
    }
    match config {
        "x" => Some(16),
        "o" => Some(8),
        "b" => Some(2),
        _ => None,
    }
}

fn binary_length(ty: &str) -> Option<usize> {
    match ty {
        "u8" | "i8" => Some(8),
        "u16" | "i16" => Some(16),
        "u32" | "i32" => Some(32),
        "u64" | "i64" => Some(64),
        "u128" | "i128" => Some(128),
        "usize" | "isize" if usize::MAX as u64 == u32::MAX as u64 => Some(32),
        "usize" | "isize" if usize::MAX as u64 == u64::MAX as u64 => Some(64),
        _ => None,
    }
}

macro_rules! get_next {
    ($iter: ident, $start: expr, $end: expr, $src: ident) => {
        if let Some(next) = $iter.next() {
            next
        } else {
            return sub_error_result(
                "Incomplete chrono format '%'. Literal '%' need to be escaped as '%%'",
                $src,
                ($start, $end),
            );
        }
    };
}

fn map_chrono_format(f: &str, src: &SscanfInner, offset: usize) -> Result<(String, String)> {
    let mut regex = String::new();
    let chrono_fmt = f.replace("\\{", "{").replace("\\}", "}");

    let mut iter = f
        .chars()
        .enumerate()
        .map(|(i, c)| (i + offset, c))
        .peekable();

    while let Some((i, c)) = iter.next() {
        if c != '%' {
            if regex_syntax::is_meta_character(c) {
                regex.push('\\');
            }
            regex.push(c);
            continue;
        }
        let mut next = get_next!(iter, i, i, src);

        let padding = match next.1 {
            '-' => Some(""),
            '0' => Some("0"),
            '_' => Some(" "),
            _ => None,
        };
        if padding.is_some() {
            next = get_next!(iter, i, next.0, src);
        }

        regex += &get_date_fmt(next, padding, src, &mut iter)?;
    }

    Ok((regex, chrono_fmt))
}

fn get_date_fmt(
    letter: (usize, char),
    padding: Option<&'static str>,
    src: &SscanfInner,
    iter: &mut impl Iterator<Item = (usize, char)>,
) -> Result<String> {
    let i = letter.0;
    let pad = |def| padding.unwrap_or(def);
    let pad_to = |def, n| {
        let padding = pad(def);
        let mut fmt = String::from("(");
        for i in 0..n {
            if i != 0 {
                fmt += "|";
            }
            for _ in 0..i {
                fmt += padding;
            }
            fmt += "[1-9]";
            for _ in (i + 1)..n {
                fmt += r"\d";
            }
        }
        fmt + ")"
    };
    Ok(match letter.1 {
        'Y' | 'G' => pad_to("0", 4),
        'C' | 'y' | 'g' => pad_to("0", 2),
        'm' => format!(r"({}\d|1[0-2])", pad("0")),
        'b' | 'h' => r"[a-zA-Z]{3}".to_string(),
        'B' => r"[a-zA-Z]+".to_string(),
        'd' => format!(r"({}\d|[12]\d|3[01])", pad("0")),
        'e' => format!(r"({}\d|[12]\d|3[01])", pad(" ")),
        'a' => r"[a-zA-Z]{3}".to_string(),
        'A' => r"[a-zA-Z]+".to_string(),
        'w' => "[0-6]".to_string(),
        'u' => "[1-7]".to_string(),
        'U' | 'W' => format!(r"({}\d|[1-4]\d|5[0-3])", pad("0")),
        'V' => format!(r"({}[1-9]|[1-4]\d|5[0-3])", pad("0")),
        'j' => format!(r"({0}{0}[1-9]|{0}\d\d|[1-3][0-5]\d|[1-3]6[0-6])", pad("0")),
        'D' => format!(
            "{}/{}/{}",
            get_date_fmt((i, 'm'), padding, src, iter)?,
            get_date_fmt((i, 'd'), padding, src, iter)?,
            get_date_fmt((i, 'y'), padding, src, iter)?
        ),
        'x' => format!(
            "{}/{}/{}",
            get_date_fmt((i, 'd'), padding, src, iter)?,
            get_date_fmt((i, 'd'), padding, src, iter)?,
            get_date_fmt((i, 'y'), padding, src, iter)?
        ),
        'F' => format!(
            "{}-{}-{}",
            get_date_fmt((i, 'Y'), padding, src, iter)?,
            get_date_fmt((i, 'm'), padding, src, iter)?,
            get_date_fmt((i, 'd'), padding, src, iter)?
        ),
        'v' => format!(
            "{}-{}-{}",
            get_date_fmt((i, 'e'), padding, src, iter)?,
            get_date_fmt((i, 'b'), padding, src, iter)?,
            get_date_fmt((i, 'Y'), padding, src, iter)?
        ),
        'H' => format!(r"({}\d|1\d|2[0-3])", pad("0")),
        'k' => format!(r"({}\d|1\d|2[0-3])", pad(" ")),
        'I' => format!(r"({}[1-9]|1[0-2])", pad("0")),
        'l' => format!(r"({}[1-9]|1[0-2])", pad(" ")),
        'P' => "(am|pm)".to_string(),
        'p' => "(AM|PM)".to_string(),
        'M' => format!(r"({}\d|[1-5]\d)", pad("0")),
        'S' => format!(r"({}\d|[1-5]\d|60)", pad("0")),
        'f' => r"\d{9}".to_string(),
        '.' => {
            let start = i - 1;
            match get_next!(iter, start, i, src) {
                (_, 'f') => r"\.\d{0,9}".to_string(),
                (ni, c @ '1'..='9') => {
                    if get_next!(iter, start, ni, src).1 == 'f' {
                        format!(r"\.\d{{{}}}", c)
                    } else {
                        return sub_error_result("Incomplete %f specifier", src, (start, ni));
                    }
                }
                _ => return sub_error_result("Incomplete %f specifier", src, (start, i)),
            }
        }
        c @ '1'..='9' => {
            let start = i - 1;
            if get_next!(iter, start, i, src).1 == 'f' {
                format!(r"\d{{{}}}", c)
            } else {
                return sub_error_result("Incomplete %f specifier", src, (start, i));
            }
        }
        'R' => format!(
            "{}:{}",
            get_date_fmt((i, 'H'), padding, src, iter)?,
            get_date_fmt((i, 'M'), padding, src, iter)?,
        ),
        'T' | 'X' => format!(
            "{}:{}:{}",
            get_date_fmt((i, 'H'), padding, src, iter)?,
            get_date_fmt((i, 'M'), padding, src, iter)?,
            get_date_fmt((i, 'S'), padding, src, iter)?,
        ),
        'r' => format!(
            "{}:{}:{} {}",
            get_date_fmt((i, 'I'), padding, src, iter)?,
            get_date_fmt((i, 'M'), padding, src, iter)?,
            get_date_fmt((i, 'S'), padding, src, iter)?,
            get_date_fmt((i, 'p'), padding, src, iter)?,
        ),
        'Z' => r"\w+".to_string(),
        'z' => r"\+\d\d\d\d".to_string(),
        c @ ':' | c @ '#' => {
            if get_next!(iter, i - 1, i, src).1 == 'z' {
                if c == ':' {
                    r"\+\d\d:\d\d".to_string()
                } else {
                    r"\+\d\d(\d\d)?".to_string()
                }
            } else {
                return sub_error_result("Incomplete %z specifier", src, (i - 1, i));
            }
        }
        'c' => format!(
            "{} {} {} {} {}",
            get_date_fmt((i, 'a'), padding, src, iter)?,
            get_date_fmt((i, 'h'), padding, src, iter)?,
            get_date_fmt((i, 'e'), padding, src, iter)?,
            get_date_fmt((i, 'X'), padding, src, iter)?,
            get_date_fmt((i, 'Y'), padding, src, iter)?,
        ),
        '+' => format!(
            r"{}T{}\.\d{{0,9}}\+\d\d:\d\d",
            get_date_fmt((i, 'F'), padding, src, iter)?,
            get_date_fmt((i, 'T'), padding, src, iter)?,
        ),
        's' => r"\d+".to_string(),
        't' => '\t'.to_string(),
        'n' => '\n'.to_string(),
        '%' => '%'.to_string(),
        x => return sub_error_result(&format!("Unknown chrono format {}", x), src, (i, i)),
    })
}

fn sub_error_result<T>(
    message: &str,
    src: &SscanfInner,
    (start, end): (usize, usize),
) -> Result<T> {
    Err(sub_error(message, src, (start, end)))
}

fn sub_error(message: &str, src: &SscanfInner, (start, end): (usize, usize)) -> Error {
    let s = start + src.span_offset + 1; // + 1 for "
    let e = end + src.span_offset + 1;
    if let Some(span) = src.fmt_span.subspan(s..=e) {
        Error::new(span, message)
    } else {
        let m = format!(
            "{}.  At \"{}\" <--",
            message,
            &src.fmt[0..=end.min(src.fmt.len() - 1)]
        );
        Error::new_spanned(&src.fmt_span, m)
    }
}

fn full_span<T: ToTokens + Spanned>(span: &T) -> (Span, Span) {
    // dirty hack stolen from syn::Error::new_spanned
    // because _once again_, spans don't really work on stable, so instead we set part of the
    // target to the beginning of the type, part to the end, and then the rust compiler joins
    // them for us. Isn't that a nice?

    let start = span.span();
    let end = span
        .to_token_stream()
        .into_iter()
        .last()
        .map(|t| t.span())
        .unwrap_or(start);
    (start, end)
}
