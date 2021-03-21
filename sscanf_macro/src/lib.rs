//! Crate with proc_macros for sscanf. Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Error, Parse, ParseStream, Result},
    parse_macro_input, Expr, LitStr, Token, TypePath,
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
        let fmt: LitStr = input.parse()?;
        let span_offset = {
            // this is a dirty hack to see if the literal is a raw string, which is necessary for
            // subspan to skip the 'r', 'r#', ...
            // this should be a thing that I can check on LitStr or Literal or whatever, but nooo,
            // I have to print it into a String via a TokenStream and check that one -.-
            let lit = fmt.to_token_stream().to_string();
            lit.char_indices().find(|(_, c)| *c == '"').unwrap().0
        };
        let mut fmt_span = Literal::string(&fmt.value());
        fmt_span.set_span(fmt.span());

        let mut type_tokens = vec![];

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                let token = input.parse()?;
                type_tokens.push(token);
            }
        }

        type_tokens.reverse();

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
        let src_str = input.parse()?;
        input.parse::<Token![,]>()?;
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
        let regex = ::sscanf::const_format!(#(#regex),*);
        ::sscanf::Regex::new(regex).unwrap()
    })
    .into()
}

fn scanf_internal(input: Sscanf, escape_input: bool) -> TokenStream1 {
    let (regex, matcher) = match generate_regex(input.inner, escape_input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };
    let src_str = input.src_str;
    quote!(
        {
            let regex = ::sscanf::const_format!( #(#regex),* );
            let regex = ::sscanf::Regex::new(regex).unwrap();
            regex.captures(&#src_str).and_then(|cap| Some(( #(#matcher),* )))
        }
    )
    .into()
}

fn generate_regex(
    mut input: SscanfInner,
    escape_input: bool,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
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

        // dirty hack stolen from syn::Error::new_spanned
        // because _once again_, spans don't really work on stable, so instead we set part of the
        // target to the beginning of the type, part to the end, and then the rust compiler joins
        // them for us. Isn't that a nice?
        let mut iter = ty.to_token_stream().into_iter();
        let start = iter.next().unwrap().span();
        let end = iter.last().map_or(start, |t| t.span());

        {
            let a = quote_spanned!(start => <#ty as );
            let b = quote_spanned!(end => ::sscanf::RegexRepresentation>::REGEX);
            regex_builder.push(quote!(#a#b));
        }
        {
            let a = quote_spanned!(start => <#ty as );
            let b = quote_spanned!(end => ::std::str::FromStr>::from_str(cap.name(#name)?.as_str()).ok()?);
            match_grabber.push(quote!(#a#b));
        }
    }

    regex_builder.push(quote!(#current_regex));

    Ok((regex_builder, match_grabber))
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
