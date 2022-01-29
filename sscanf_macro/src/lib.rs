//! Crate with proc_macros for sscanf. Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Literal, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input,
    spanned::Spanned,
    Expr, LitStr, Path, Token,
};

mod chrono;
mod format_config;

struct PlaceHolder {
    name: String,
    type_token: Option<Path>,
    config: Option<String>,
    span: (usize, usize),
}

/// Format string and types for scanf_get_regex. Shared by scanf and scanf_unescaped
struct ScanfInner {
    /// content of the format string
    fmt: String,
    /// subspan-provider for the format string
    fmt_span: Literal,
    /// number of chars in fmt_span before content starts (e.g. 2 for "r#")
    span_offset: usize,
    /// Types after the format string
    type_tokens: Vec<Path>,
}
/// Input string, format string and types for scanf and scanf_unescaped
struct Scanf {
    /// input to run the scanf on
    src_str: Expr,
    /// format string and types
    inner: ScanfInner,
}

impl Parse for ScanfInner {
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
            // "fun" fact: This used to be easier in syn 0.11 where `Lit::Str` gave _two_ values:
            // the literal and a `StrStyle` which looked like this:
            // enum StrStyle {
            //     Cooked,      // a non-raw string
            //     Raw(usize),
            // }       ^^^^^ See this usize here? That's the number of '#' in the prefix
            //               which is _exactly_ what I'm trying to calculate here! How convenient!
            // This was apparently removed at some point for being TOO USEFUL.
            let lit = fmt.to_token_stream().to_string();
            // Yes, this is the easiest way to solve this. Have syn read the Rust Code (which used
            // to be a string) as a TokenStream to parse the LitStr, turn that back into a
            // TokenStream and then into a String (LitStr cannot be directly converted to a String)
            // and then iterate over that string for the first ", because anything before that
            // **should** (this will totally break at some point) be the prefix.
            lit.chars().position(|c| c == '"').unwrap()
        };

        // fmt has to be parsed as `syn::LitStr` to access the content as a string. But in order to
        // call subspan, we need it as a `proc_macro2::Literal`. So: parse it as `LitStr` first and
        // convert that to a `Literal` with the same content and span.
        let mut fmt_span = Literal::string(&fmt.value());
        fmt_span.set_span(fmt.span()); // fmt is a single Token so span() works even on stable

        let type_tokens;
        if input.is_empty() {
            type_tokens = vec![];
        } else {
            input.parse::<Token![,]>()?; // the comma after the format string

            type_tokens = input
                .parse_terminated::<_, Token![,]>(Path::parse)?
                .into_iter()
                .collect();
        }

        Ok(ScanfInner {
            fmt: fmt.value(),
            fmt_span,
            span_offset,
            type_tokens,
        })
    }
}
impl Parse for Scanf {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            // All of these special cases have to be handled separately, because syn's default
            // behavior when something is missing is to point at the entire macro invocation with
            // an error message that says "expected <missing thing>". But if a user sees the entire
            // thing underlined with the message "expected a comma", they will assume that they
            // should replace that macro call with a comma or something similar. They would not
            // guess that the actual meaning is:
            // "this macro requires more parameters than I have given it, and the next
            // parameter should be separated with a comma from the current ones which is why the
            // macro expected a comma, and it would point to the end of the input where the comma
            // was expected, but since there is nothing there it has no span to point to so it
            // just points at the entire thing."
            // I love writing error messages in proc macros :D (not)
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
            // Addition to the comment above: here we actually have a comma to point to to say:
            // "Hey, you put a comma here, put something after it". syn doesn't do this
            // because it can't rewind the input stream to check this.
            return Err(Error::new_spanned(
                comma,
                "At least 2 Parameters required: Missing format string",
            ));
        }
        let inner = input.parse()?;

        Ok(Scanf { src_str, inner })
    }
}

#[proc_macro]
pub fn scanf(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Scanf);
    scanf_internal(input, true)
}

#[proc_macro]
pub fn scanf_unescaped(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Scanf);
    scanf_internal(input, false)
}

#[proc_macro]
pub fn scanf_get_regex(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as ScanfInner);
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

fn scanf_internal(input: Scanf, escape_input: bool) -> TokenStream1 {
    let (regex, matcher) = match generate_regex(input.inner, escape_input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let src_str = {
        let src_str = input.src_str;
        let (start, end) = full_span(&src_str);
        let mut param = quote_spanned!(start => &);
        param.extend(quote_spanned!(end => (#src_str)));
        // wrapping the input in a manual call to AsRef::as_ref ensures that the user
        // gets an appropriate error message if they try to use a non-string input
        quote!(::std::convert::AsRef::<str>::as_ref(#param))
    };
    quote!(
        {
            #regex
            let input = #src_str;
            #[allow(clippy::needless_question_mark)]
            REGEX.captures(input)
                .ok_or_else(|| ::sscanf::Error::RegexMatchFailed(input, &REGEX))
                .and_then(|cap| Ok(( #(#matcher),* )))
        }
    )
    .into()
}

fn generate_regex(
    input: ScanfInner,
    escape_input: bool,
) -> Result<(TokenStream, Vec<TokenStream>)> {
    let (mut placeholders, regex_parts) = parse_format_string(&input, escape_input)?;

    let mut type_tokens = input.type_tokens.iter().cloned();

    let mut error = TokenStream::new();
    for ph in &mut placeholders {
        if ph.type_token.is_none() {
            if let Some(ty) = type_tokens.next() {
                ph.type_token = Some(ty);
            } else {
                // generate an error for all placeholders that don't have a corresponding type
                let message = if let Some(config) = &ph.config {
                    format!("Missing Type for given '{{:{}}}' Placeholder", config)
                } else {
                    "Missing Type for given '{}' Placeholder".to_string()
                };
                error.extend(sub_error(&message, &input, ph.span).to_compile_error());
            }
        }
    }
    // generate an error for all types that don't have a corresponding placeholder
    for ty in type_tokens {
        error.extend(
            Error::new_spanned(ty, "More Types than '{}' Placeholders provided").to_compile_error(),
        );
    }

    if !error.is_empty() {
        error.extend(quote!(let REGEX = ::sscanf::regex::Regex::new("").unwrap();));
        return Ok((error, vec![]));
    }

    // these need to be Vec instead of TokenStream to allow adding the comma separators later
    let mut regex_builder = vec![];
    let mut match_grabber = vec![];
    // if there are n types, there are n+1 regex_parts, so add the first n during this loop and
    // add the last one afterwards
    for (ph, regex_prefix) in placeholders.iter().zip(regex_parts.iter()) {
        let ty = ph.type_token.as_ref().unwrap();

        regex_builder.push(quote!(#regex_prefix));
        let mut regex = None;
        let mut converter = None;

        if let Some(config) = ph.config.as_ref() {
            let res = format_config::regex_from_config(config, ty, ph, &input)?;
            regex = Some(res.0);
            converter = res.1;
        }

        let (start, end) = full_span(&ty);
        let name = &ph.name;

        let regex = regex.unwrap_or_else(|| {
            // proc_macros don't have any type information, so we can't check if the type
            // implements the trait, so we wrap it in this verbose <#ty as Trait> code,
            // so that the compiler can check if the trait is implemented, and, most importantly,
            // tell the user if they forgot to implement the trait.
            // The code is split into two quote_spanned calls in case the type consists of more
            // than one token (like std::vec::Vec). Again, no span manipulation on stable and we
            // obviously want the entire type underlined, so we have to map the start and end of
            // the type's span to the start and end of the part that generates the error message.
            // Yes, this works. No, this is not a good way to do this. ¯\_(ツ)_/¯
            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::sscanf::RegexRepresentation>::REGEX));
            s
        });
        regex_builder.push(regex);

        let converter = converter.unwrap_or_else(|| {
            let start_convert = quote_spanned!(start => <#ty as );
            let end_convert = quote_spanned!(end => ::std::str::FromStr>::from_str(input));
            quote!(#start_convert #end_convert)
        });

        let matcher = quote!({
            let input = cap.name(#name)
                .expect("scanf: Invalid regex: Could not find one of the captures")
                .as_str();
            #converter
                .map_err(|err| ::sscanf::Error::FromStrFailed(stringify!(#ty), input, Box::new(err)))?
        });
        match_grabber.push(matcher);
    }

    // add the last regex_part
    let last_regex = &regex_parts[placeholders.len()];
    regex_builder.push(quote!(#last_regex));

    #[rustfmt::skip]
    let regex = quote!(
        ::sscanf::lazy_static::lazy_static!{
            static ref REGEX: ::sscanf::regex::Regex =
                ::sscanf::regex::Regex::new(
                    ::sscanf::const_format::concatcp!( #(#regex_builder),* )
                )
                .expect("scanf: cannot generate Regex");
        }
    );

    Ok((regex, match_grabber))
}

fn parse_format_string(
    input: &ScanfInner,
    escape_input: bool,
) -> Result<(Vec<PlaceHolder>, Vec<String>)> {
    let mut placeholders = vec![];

    // all completed parts of the regex
    let mut regex = vec![];
    let mut current_regex = String::from("^");

    // name of the next placeholder
    let mut name_index = 1;

    // keep the iterator as a variable to allow peeking and advancing in a sub-function
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
            } else {
                // escaped '{{', will be handled like a regular char by the following code
            }
        } else if c == '}' {
            if iter.next() == Some((i + 1, '}')) {
                // escaped '}}', will be handled like a regular char by the following code
                // next automatically advanced the iterator to skip the second '}'
            } else {
                // we have a '}' that is not escaped and not in a placeholder
                return sub_error_result(
                    "Unexpected standalone '}'. Literal '}' need to be escaped as '}}'",
                    input,
                    (i, i),
                );
            }
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

/// Returns a span inside of fmt_span, if possible. Otherwise, returns the entire span.
fn sub_span(src: &ScanfInner, (start, end): (usize, usize)) -> Span {
    let s = start + src.span_offset + 1; // + 1 for "
    let e = end + src.span_offset + 1;
    src.fmt_span
        .subspan(s..=e)
        .unwrap_or_else(|| src.fmt_span.span())
}

/// `sub_error`, but wrapped in a Result.
fn sub_error_result<T>(message: &str, src: &ScanfInner, (start, end): (usize, usize)) -> Result<T> {
    Err(sub_error(message, src, (start, end)))
}

/// Generates an error for a subsection of the format string
fn sub_error(message: &str, src: &ScanfInner, (start, end): (usize, usize)) -> Error {
    let s = start + src.span_offset + 1; // + 1 for "
    let e = end + src.span_offset + 1;

    // subspan allows pointing at a span that is not the whole string, but it only works in nightly
    if let Some(span) = src.fmt_span.subspan(s..=e) {
        Error::new(span, message)
    } else {
        // Workaround for stable: print a copy of the entire format string into the error message
        // and manually underline the desired section.
        let mut m = format!("{}:", message);
        m.push('\n');

        // Add the line with the format string
        if src.span_offset > 0 {
            let hashtags = (1..src.span_offset).map(|_| '#').collect::<String>();

            m += &format!("At r{0}\"{1}\"{0}", hashtags, src.fmt);
        } else {
            m += &format!("At \"{}\"", src.fmt);
        }
        m.push('\n');

        // Add the line with the error squiggles
        // s already includes the span_offset and +1 for the ", so only the "At " prefix is missing
        for _ in 0..("At ".len() + s) {
            m.push(' ');
        }
        for _ in s..=e {
            m.push('^');
        }
        Error::new_spanned(&src.fmt_span, m)
    }
}

fn full_span<T: ToTokens + Spanned>(span: &T) -> (Span, Span) {
    // dirty hack stolen from syn::Error::new_spanned
    // because _once again_, spans don't really work on stable, so instead we set part of the
    // target to the beginning of the type, part to the end, and then the rust compiler joins
    // them for us.

    let start = span.span();
    let end = span
        .to_token_stream()
        .into_iter()
        .last()
        .map(|t| t.span())
        .unwrap_or(start);
    (start, end)
}
