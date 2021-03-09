//! Crate with proc_macros for sscanf. Not usable as a standalone crate.

#[macro_use]
extern crate quote;

use proc_macro::{TokenStream, TokenTree};
use syn::*;

macro_rules! proc_panic {
    ($message: expr) => {{
        let m = $message;
        return quote!(compile_error!(#m)).into()
    }};
    ($message: expr, $span: expr) => {{
        let m = $message;
        return quote_spanned!($span.into() => compile_error!(#m)).into()
    }};
    ($message: expr, $span_src: expr, $start: expr, $end: expr, $src: expr) => {{
        if let Some(span) = $span_src.subspan($start..=$end) {
            proc_panic!($message, span);
        } else {
            let m = format!("{}.  At \"{}\" <--", $message, &$src[0..$end]);
            proc_panic!(m, $span_src.span());
        }
    }};
}

fn split_at_comma(stream: TokenStream) -> Vec<TokenStream> {
    let mut ret = vec![];
    let mut current = vec![];
    for token in stream {
        match token {
            TokenTree::Punct(p)
                if p.as_char() == ',' && p.spacing() == proc_macro::Spacing::Alone =>
            {
                ret.push(current.into_iter().collect());
                current = vec![];
            }
            tt => current.push(tt),
        }
    }
    if !current.is_empty() {
        ret.push(current.into_iter().collect());
    }
    ret
}

#[proc_macro]
pub fn scanf(input: TokenStream) -> TokenStream {
    scanf_internal(input, true, false)
}

#[proc_macro]
pub fn scanf_get_regex(input: TokenStream) -> TokenStream {
    scanf_internal(input, true, true)
}

#[proc_macro]
pub fn scanf_unescaped(input: TokenStream) -> TokenStream {
    scanf_internal(input, false, false)
}

fn scanf_internal(input: TokenStream, escape_input: bool, return_regex: bool) -> TokenStream {
    let mut tokens = split_at_comma(input);
    if tokens.len() < 2 {
        proc_panic!("scanf needs at least an input and a format string");
    }
    tokens.reverse();

    let src_string = if return_regex {
        proc_macro2::TokenStream::new()
    } else {
        proc_macro2::TokenStream::from(tokens.pop().unwrap())
    };

    let fmt = {
        let stream = tokens.last().unwrap().clone();
        parse_macro_input!(stream as LitStr).value()
    };
    let fmt_literal = match tokens.pop().unwrap().into_iter().next().unwrap() {
        TokenTree::Literal(lit) => lit,
        tt => proc_panic!("Expected string literal", tt.span()),
    };
    let mut span_provider = proc_macro2::Literal::string(&fmt);
    span_provider.set_span(fmt_literal.span().into());

    let mut type_tokens = vec![];
    let mut type_token_names = vec![];
    let mut regex = vec![];

    let mut name_index = 1;

    let mut current_regex = String::from("^");
    let mut last_was_open = false;
    let mut last_was_close = false;
    for (i, c) in fmt.chars().enumerate().map(|(i, c)| (i + 1, c)) {
        if c == '{' && !last_was_close {
            if last_was_open {
                last_was_open = false;
            } else {
                last_was_open = true;
                continue;
            }
        } else if c == '}' {
            if last_was_open {
                let ty = if let Some(token) = tokens.pop() {
                    parse_macro_input!(token as ExprPath)
                } else {
                    proc_panic!("Missing Type for given '{}'", span_provider, i - 1, i, fmt);
                };
                type_tokens.push(ty.clone());

                let name = format!("type_{}", name_index);
                name_index += 1;
                current_regex += &format!("(?P<{}>", name);
                type_token_names.push(name);

                regex.push(current_regex);
                current_regex = String::from(")");

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
            proc_panic!(
                "Expected '}' after '{'. Literal '{' need to be escaped as '{{'",
                span_provider,
                i - 1,
                i - 1,
                fmt
            );
        } else if last_was_close {
            proc_panic!(
                "Unexpected standalone '}'. Literal '}' need to be escaped as '}}'",
                span_provider,
                i - 1,
                i - 1,
                fmt
            );
        }

        if escape_input && regex_syntax::is_meta_character(c) {
            current_regex.push('\\');
        }

        current_regex.push(c);
    }

    current_regex.push('$');

    let len = fmt.len();
    if last_was_open {
        proc_panic!(
            "Expected '}' after '{'. Literal '{' need to be escaped as '{{'",
            span_provider,
            len,
            len,
            fmt
        );
    } else if last_was_close {
        proc_panic!(
            "Unexpected standalone '}'. Literal '}' need to be escaped as '}}'",
            span_provider,
            len,
            len,
            fmt
        );
    }

    if let Some(token) = tokens.pop() {
        proc_panic!(
            "More Types than '{}' provided",
            token.into_iter().next().unwrap().span()
        );
    }

    if type_tokens.is_empty() {
        if return_regex {
            proc_panic!("Cannot generate Regex without Type Parameters");
        }
        return quote!( if #src_string.starts_with(#fmt) { Some(()) } else { None } ).into();
    }

    let mut regex_builder = vec![];
    let mut match_grabber = vec![];
    for (prefix, (ty, name)) in regex.iter().zip(type_tokens.iter().zip(type_token_names)) {
        regex_builder.push(quote!(#prefix));
        regex_builder.push(quote!(<#ty as ::sscanf::RegexRepresentation>::REGEX));
        match_grabber.push(quote!(cap.name(#name)?.as_str().parse::<#ty>().ok()?))
    }

    if return_regex {
        return quote!({
            let regex = ::sscanf::const_format!(#(#regex_builder),*, #current_regex);
            ::sscanf::Regex::new(regex).unwrap()
        })
        .into();
    }

    quote!(
        {
            let regex = ::sscanf::const_format!(#(#regex_builder),*, #current_regex);
            let regex = ::sscanf::Regex::new(regex).unwrap();
            regex.captures(#src_string).and_then(|cap| Some(( #(#match_grabber),* )))
        }
    )
    .into()
}
