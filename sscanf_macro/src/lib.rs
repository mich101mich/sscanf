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
            lit.char_indices().find(|c| c.1 == '"').unwrap().0
        };
        let mut fmt_span = Literal::string(&fmt.value());
        fmt_span.set_span(fmt.span());

        let type_tokens;
        if input.is_empty() {
            type_tokens = vec![];
        } else {
            input.parse::<Token![,]>()?;

            type_tokens = input
                .parse_terminated::<_, Token![,]>(TypePath::parse)?
                .into_iter()
                .rev()
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
    mut input: SscanfInner,
    escape_input: bool,
) -> Result<(TokenStream, Vec<TokenStream>)> {
    let mut type_tokens = vec![];
    let mut regex = vec![];

    let mut name_index = 1;

    let mut current_regex = String::from("^");
    let mut last_was_open = false;
    let mut last_was_close = false;
    for (i, c) in input.fmt.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        if c == '{' && !last_was_close {
            if last_was_open {
                last_was_open = false;
            } else {
                last_was_open = true;
                continue;
            }
        } else if c == '}' {
            if last_was_open {
                let ty = match input.type_tokens.pop() {
                    Some(token) => token,
                    None => return sub_error("Missing Type for given '{}'", &input, i - 1, i),
                };

                let name = format!("type_{}", name_index);
                name_index += 1;

                current_regex += &format!("(?P<{}>", name);
                regex.push(current_regex);
                current_regex = String::from(")");

                type_tokens.push((ty, name));

                last_was_open = false;
                continue;
            }
            if last_was_close {
                last_was_close = false;
            } else {
                last_was_close = true;
                continue;
            }
        } else if last_was_open {
            return sub_error(
                "Expected '}' after '{'. Literal '{' need to be escaped as '{{'",
                &input,
                i - 1,
                i - 1,
            );
        } else if last_was_close {
            return sub_error(
                "Unexpected standalone '}'. Literal '}' need to be escaped as '}}'",
                &input,
                i - 1,
                i - 1,
            );
        }

        if escape_input && regex_syntax::is_meta_character(c) {
            current_regex.push('\\');
        }

        current_regex.push(c);
    }

    current_regex.push('$');

    let end = input.fmt.len();
    if last_was_open {
        return sub_error(
            "Expected '}' after '{'. Literal '{' need to be escaped as '{{'",
            &input,
            end,
            end,
        );
    } else if last_was_close {
        return sub_error(
            "Unexpected standalone '}'. Literal '}' need to be escaped as '}}'",
            &input,
            end,
            end,
        );
    }

    if let Some(token) = input.type_tokens.pop() {
        return Err(Error::new_spanned(token, "More Types than '{}' provided"));
    }

    let mut regex_builder = vec![];
    let mut match_grabber = vec![];
    for (prefix, (ty, name)) in regex.iter().zip(type_tokens) {
        regex_builder.push(quote!(#prefix));

        let (start, end) = full_span(&ty);

        {
            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::sscanf::RegexRepresentation>::REGEX));
            regex_builder.push(s);
        }
        {
            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::std::str::FromStr>::from_str(cap.name(#name)?.as_str()).ok()?));
            match_grabber.push(s);
        }
    }

    regex_builder.push(quote!(#current_regex));

    let regex = quote!(::sscanf::lazy_static::lazy_static! {
        static ref REGEX: ::sscanf::regex::Regex = ::sscanf::regex::Regex::new(
            ::sscanf::const_format::concatcp!( #(#regex_builder),* )
        ).expect("sscanf Regex error");
    });

    Ok((regex, match_grabber))
}

fn sub_error<T>(message: &str, src: &SscanfInner, start: usize, end: usize) -> Result<T> {
    let s = start + src.span_offset;
    let e = end + src.span_offset;
    if let Some(span) = src.fmt_span.subspan(s..=e) {
        Err(Error::new(span, message))
    } else {
        let m = format!("{}.  At \"{}\" <--", message, &src.fmt[0..end]);
        Err(Error::new_spanned(&src.fmt_span, m))
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
