#![deny(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
//
// set of clippy pedantic lints that I disagree with
#![allow(
    clippy::wildcard_imports,
    clippy::enum_glob_use,
    clippy::manual_assert, // I don't want the "assertion failed" text in the panic message
    clippy::items_after_statements // if an item is only used locally, define it where it is needed
)]
//
//! Procedural macros for the [`sscanf`](https://crates.io/crates/sscanf) crate. Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
pub(crate) use proc_macro2::{Span, TokenStream};
pub(crate) use quote::{ToTokens, quote, quote_spanned};
pub(crate) use syn::{
    Error, Result, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

mod attribute;
mod error;
mod format_string;
mod sequence_matcher;
mod str_lit;
mod ty;
mod utils;

pub(crate) use attribute::*;
pub(crate) use error::*;
pub(crate) use format_string::*;
pub(crate) use sequence_matcher::*;
pub(crate) use str_lit::*;
pub(crate) use ty::*;
pub(crate) use utils::*;

mod derive;

/// Input string, format string, and types for `sscanf` and `sscanf_with_regex`.
struct Sscanf {
    /// input to run the `sscanf` on
    input: syn::Expr,
    /// format string and types
    parser: SscanfParser,
}

struct SscanfParser {
    /// the format string
    fmt: StrLit,
    /// Types after the format string
    type_tokens: Vec<Type<'static>>,
}

impl Parse for Sscanf {
    fn parse(tokens: ParseStream) -> Result<Self> {
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
        assert_or_bail!(!tokens.is_empty(), Span::call_site() => "sscanf: at least 2 Parameters required: Input and format string");

        let input: syn::Expr = tokens.parse()?;
        assert_or_bail!(!tokens.is_empty(), input.end_span() => "sscanf: at least 2 Parameters required: Missing format string");

        let comma = tokens.parse::<Token![,]>()?;
        // Addition to the comment above: here we actually have a comma to point to to say:
        // "Hey, you put a comma here, put something after it". syn doesn't do this
        // because it cannot rewind the input stream to check this.
        assert_or_bail!(!tokens.is_empty(), comma.end_span() => "at least 2 Parameters required: Missing format string");

        let parser = tokens.parse::<SscanfParser>()?;

        Ok(Sscanf { input, parser })
    }
}

impl Parse for SscanfParser {
    fn parse(tokens: ParseStream) -> Result<Self> {
        assert_or_bail!(!tokens.is_empty(), Span::call_site() => "sscanf_parser requires at least a format string");

        let fmt = tokens.parse::<StrLit>()?;

        let type_tokens = if tokens.is_empty() {
            vec![]
        } else {
            tokens.parse::<Token![,]>()?; // the comma after the format string

            tokens
                .parse_terminated(Type::parse, Token![,])?
                .into_iter()
                .collect()
        };

        Ok(SscanfParser { fmt, type_tokens })
    }
}

#[proc_macro]
pub fn sscanf(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as Sscanf);
    sscanf_internal(input, true).into_token_stream_1()
}

#[proc_macro]
pub fn sscanf_with_regex(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as Sscanf);
    sscanf_internal(input, false).into_token_stream_1()
}

#[proc_macro]
pub fn sscanf_parser(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as SscanfParser);
    sscanf_parser_internal(&input, true).into_token_stream_1()
}

#[proc_macro]
pub fn sscanf_parser_with_regex(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as SscanfParser);
    sscanf_parser_internal(&input, false).into_token_stream_1()
}

#[proc_macro_derive(FromScanf, attributes(sscanf))]
pub fn derive_from_sscanf(input: TokenStream1) -> TokenStream1 {
    let syn::DeriveInput {
        ident,
        generics,
        data,
        attrs,
        ..
    } = syn::parse_macro_input!(input as syn::DeriveInput);

    let res = match data {
        syn::Data::Struct(data) => derive::parse_struct(&ident, &generics, attrs, data),
        syn::Data::Enum(data) => derive::parse_enum(&ident, &generics, attrs, data),
        syn::Data::Union(data) => derive::parse_union(&ident, &generics, attrs, data),
    };
    match res {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

/// Internal function implementing the `sscanf` and `sscanf_with_regex` macros.
fn sscanf_internal(input: Sscanf, escape_input: bool) -> Result<TokenStream> {
    let parser = sscanf_parser_internal(&input.parser, escape_input)?;

    let src_str = {
        let start_span = input.input.span().stable_start();
        let mut src_str = quote_spanned! {start_span=> &};
        input.input.to_tokens(&mut src_str);
        src_str
    };

    let ret = quote! { #parser.parse(#src_str) };
    Ok(ret)
}

/// Internal function to generate a `Parser` from `SscanfParser`.
fn sscanf_parser_internal(input: &SscanfParser, escape_input: bool) -> Result<TokenStream> {
    let format = FormatString::new(input.fmt.to_slice(), escape_input)?;

    // inner function to use early return. This should be a closure, but those can't have lifetimes
    fn find_ph_type<'a>(
        ph: &Placeholder<'a>,
        visited: &mut [bool],
        ph_index: &mut usize,
        external_types: &[Type<'a>],
    ) -> Result<Type<'a>> {
        let n = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text().parse::<usize>() {
                assert_or_bail!(n < visited.len(), name => "type index {} out of range of {} types", n, visited.len());
                n
            } else {
                return Type::from_str(*name).map_err(|err| {
                    let hint =  "The syntax for placeholders is {<type>} or {<type>:<config>}. Make sure <type> is a valid type or index.";
                    let hint2 = "If you want syntax highlighting and better errors, place the type in the arguments after the format string while debugging";
                    let msg = format!("invalid type in placeholder: {err}.\nHint: {hint}\n{hint2}");
                    name.error(msg)
                });
            }
        } else {
            let n = *ph_index;
            *ph_index += 1;
            assert_or_bail!(n < visited.len(), ph => "more placeholders than types provided");
            n
        };
        visited[n] = true;
        Ok(external_types[n].clone())
    }

    let mut ph_index = 0;
    let mut visited = vec![false; input.type_tokens.len()];
    let mut types = vec![];
    let mut error = ErrorBuilder::new();

    for ph in &format.placeholders {
        match find_ph_type(ph, &mut visited, &mut ph_index, &input.type_tokens) {
            Ok(ty) => types.push(ty),
            Err(e) => error.push(e),
        }
    }

    for (visited, ty) in visited.iter().zip(&input.type_tokens) {
        if !*visited {
            error.with_spanned(ty, "unused type");
        }
    }

    error.ok_or_build()?;

    let sequence_matcher = SequenceMatcher::new(&format, &types, escape_input);

    let matcher = sequence_matcher.get_matcher();
    let expected_parts = sequence_matcher.num_parts();
    let parsers = sequence_matcher.parsers;
    let ret = quote! {
        ::sscanf::Parser::from_matcher(
            #matcher,
            |src| {
                let src = src.as_seq();
                assert_eq!(src.num_children(), #expected_parts, "sscanf: internal error: unexpected number of parts");

                #[allow(unused_parens, reason = "The code is autogenerated, so it can't check if it could be simplified")]
                #[allow(clippy::needless_question_mark, reason = "The code is autogenerated, so it can't check if it could be simplified")]
                #[allow(clippy::double_parens, reason = "The code is autogenerated, so it can't check if it could be simplified")]
                ::std::option::Option::Some(( #(#parsers),* ))
            }
        )
    };
    Ok(ret)
}
