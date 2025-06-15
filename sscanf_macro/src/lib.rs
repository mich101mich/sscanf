//! Procedural macros for the [`sscanf`](https://crates.io/crates/sscanf) crate. Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
pub(crate) use proc_macro2::{Span, TokenStream};
pub(crate) use quote::{quote, ToTokens};
pub(crate) use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Token,
};

mod attribute;
mod error;
mod format_option;
mod format_string;
mod placeholder;
mod regex_parts;
mod str_lit;
mod ty;
mod utils;

pub(crate) use attribute::*;
pub(crate) use error::*;
pub(crate) use format_option::*;
pub(crate) use format_string::*;
pub(crate) use placeholder::*;
pub(crate) use regex_parts::*;
pub(crate) use str_lit::*;
pub(crate) use ty::*;
pub(crate) use utils::*;

mod derive;

/// Format string and types for `sscanf_get_regex`. Shared by `sscanf` and `sscanf_unescaped`
struct ScanfInner {
    /// the format string
    fmt: StrLit,
    /// Types after the format string
    type_tokens: Vec<Type<'static>>,
}
/// Input string, format string and types for `sscanf` and `sscanf_unescaped`
struct Scanf {
    /// input to run the `sscanf` on
    src_str: syn::Expr,
    /// format string and types
    inner: ScanfInner,
}

impl Parse for ScanfInner {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            let msg = "missing parameter: format string";
            return Err(syn::Error::new(Span::call_site(), msg)); // checked in tests/fail/missing_params.rs
        }

        let fmt = input.parse::<StrLit>()?;

        let type_tokens = if input.is_empty() {
            vec![]
        } else {
            input.parse::<Token![,]>()?; // the comma after the format string

            input
                .parse_terminated(Type::parse, Token![,])?
                .into_iter()
                .collect()
        };

        Ok(ScanfInner { fmt, type_tokens })
    }
}
impl Parse for Scanf {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
            let msg = "at least 2 Parameters required: Input and format string";
            return Err(syn::Error::new(Span::call_site(), msg)); // checked in tests/fail/missing_params.rs
        }
        let src_str = input.parse()?;
        if input.is_empty() {
            let msg = "at least 2 Parameters required: Missing format string";
            return Err(syn::Error::new_spanned(src_str, msg)); // checked in tests/fail/missing_params.rs
        }
        let comma = input.parse::<Token![,]>()?;
        if input.is_empty() {
            // Addition to the comment above: here we actually have a comma to point to to say:
            // "Hey, you put a comma here, put something after it". syn doesn't do this
            // because it cannot rewind the input stream to check this.
            let msg = "at least 2 Parameters required: Missing format string";
            return Err(syn::Error::new_spanned(comma, msg)); // checked in tests/fail/missing_params.rs
        }
        let inner = input.parse()?;

        Ok(Scanf { src_str, inner })
    }
}

#[proc_macro]
pub fn sscanf(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as Scanf);
    sscanf_internal(input).into_proc_macro_token_stream()
}

#[proc_macro]
pub fn sscanf_parser(input: TokenStream1) -> TokenStream1 {
    let input = syn::parse_macro_input!(input as ScanfInner);
    generate_parser(&input).into_proc_macro_token_stream()
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
        Err(err) => err.into(),
    }
}

fn sscanf_internal(input: Scanf) -> Result<TokenStream> {
    let parser = generate_parser(&input.inner)?;
    let input = input.src_str;
    // TODO: wrap in OnceLock or similar
    Ok(quote! { #parser.parse(&#input) })
}

fn generate_parser(input: &ScanfInner) -> Result<TokenStream> {
    let mut format = FormatString::new(input.fmt.to_slice(), input.fmt.is_raw())?;
    format.parts[0].insert(0, '^');
    format.parts.last_mut().unwrap().push('$');

    // inner function to have early returns. This should be a closure, but those can't have lifetimes
    fn find_ph_type<'a>(
        ph: &Placeholder<'a>,
        visited: &mut [bool],
        ph_index: &mut usize,
        external_types: &[Type<'a>],
    ) -> Result<Type<'a>> {
        let n = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text().parse::<usize>() {
                if n >= visited.len() {
                    let msg = format!("type index {n} out of range of {} types", visited.len());
                    return name.err(&msg); // checked in tests/fail/<channel>/invalid_type_in_placeholder.rs
                }
                n
            } else {
                return Type::from_str(name.clone()).map_err(|err| {
                    let hint =  "The syntax for placeholders is {<type>} or {<type>:<config>}. Make sure <type> is a valid type or index.";
                    let hint2 = "If you want syntax highlighting and better errors, place the type in the arguments after the format string while debugging";
                    let msg = format!("invalid type in placeholder: {err}.\nHint: {hint}\n{hint2}");
                    name.error(msg) // checked in tests/fail/<channel>/invalid_type_in_placeholder.rs
                });
            }
        } else {
            let n = *ph_index;
            *ph_index += 1;
            if n >= visited.len() {
                let msg = "more placeholders than types provided";
                return ph.src.err(msg); // checked in tests/fail/<channel>/missing_type.rs
            }
            n
        };
        visited[n] = true;
        Ok(external_types[n].clone())
    }

    let mut ph_index = 0;
    let mut visited = vec![false; input.type_tokens.len()];
    let mut types = vec![];
    let mut parser_vars = vec![];
    let mut error = Error::builder();

    for (i, ph) in format.placeholders.iter().enumerate() {
        match find_ph_type(ph, &mut visited, &mut ph_index, &input.type_tokens) {
            Ok(ty) => {
                types.push(ty);
                parser_vars.push(syn::Ident::new(&format!("p{i}"), Span::call_site()));
            }
            Err(e) => error.push(e),
        }
    }

    for (visited, ty) in visited.iter().zip(&input.type_tokens) {
        if !*visited {
            error.with_spanned(ty, "unused type"); // checked in tests/fail/missing_placeholder.rs
        }
    }

    error.ok_or_build()?;

    let regex_parts = RegexParts::new(&format, &types)?;

    let parsers: Vec<TokenStream> = vec![];
    let ret = quote! {{
        type ScanfOutputTuple = ( #( #types ),* );
        struct ScanfOutputWrapper<'input>(
            #( #types, )*
            ::std::marker::PhantomData<&'input ()> // in case none of the types use the lifetime
        );
        struct ScanfOutputParser<'input>(
            #( ::sscanf::from_scanf::SubType<'input, #types>, )*
            ::std::marker::PhantomData<&'input ()>
        );

        impl<'input> ::sscanf::FromScanf<'input> for ScanfOutputWrapper<'input> {
            type Parser = ScanfOutputParser<'input>;
            fn create_parser(_: &::sscanf::from_scanf::format_options::FormatOptions) -> (::sscanf::from_scanf::RegexSegment, Self::Parser) {
                todo!()
            }
        }

        impl<'input> ::std::convert::From<ScanfOutputWrapper<'input>> for ScanfOutputTuple {
            fn from(wrapper: ScanfOutputWrapper<'input>) -> Self {
                let ScanfOutputWrapper( #( #parser_vars, )* _) = wrapper;
                ( #( #parser_vars ),* )
            }
        }

        impl<'input> ::sscanf::from_scanf::FromScanfParser<'input, ScanfOutputWrapper<'input>> for ScanfOutputParser<'input> {
            fn parse(
                &self,
                _: &'input ::std::primitive::str,
                mut sub_matches: &[::std::option::Option<&'input ::std::primitive::str>]
            ) -> ::std::option::Option<ScanfOutputWrapper<'input>> {
                let ScanfOutputParser( #( #parser_vars, )* _) = self;

                #( let #parser_vars = #parser_vars.parse(&mut sub_matches)?; )*

                assert!(sub_matches.is_empty(), "sscanf: expected no remaining sub-matches, got {}. Are there any custom types with incorrect FromScanf implementations?", sub_matches.len());

                Some(ScanfOutputWrapper( #( #parser_vars, )* ::std::marker::PhantomData))
            }
        }

        ::sscanf::parser_object::Sscanf::<ScanfOutputTuple>::new::<ScanfOutputWrapper>()
    }};

    Ok(ret)
}
