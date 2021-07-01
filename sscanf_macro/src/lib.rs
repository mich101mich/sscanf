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

mod chrono;
mod format_config;

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
            let (regex, matcher) = format_config::regex_from_config(config, ty, ph, &input)?;
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
            if let Some(mut ph) = format_config::parse_bracket_content(&mut iter, input, i)? {
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
                    input,
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
        let mut m = format!("{}:\nAt ", message);
        let msg_len = 3; // "At "

        if src.span_offset > 0 {
            m.push('r');
            (1..src.span_offset).for_each(|_| m.push('#'));
        }
        m.push('"');
        m += &src.fmt;
        m.push('"');
        (1..src.span_offset).for_each(|_| m.push('#'));
        m.push('\n');
        (0..(s + msg_len)).for_each(|_| m.push(' '));
        (s..=e).for_each(|_| m.push('^'));
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
