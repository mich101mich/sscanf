//! Crate with proc_macros for [sscanf](https://crates.io/crates/sscanf). Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    DeriveInput, Expr, Path, Token, Type, TypePath,
};

mod attributes;
mod error;
mod format_option;
mod format_string;
mod placeholder;
mod regex_parts;
mod str_lit;

pub(crate) use attributes::*;
pub(crate) use error::*;
pub(crate) use format_option::*;
pub(crate) use format_string::*;
pub(crate) use placeholder::*;
pub(crate) use regex_parts::*;
pub(crate) use str_lit::*;

mod derive;

/// Format string and types for sscanf_get_regex. Shared by sscanf and sscanf_unescaped
struct ScanfInner {
    /// the format string
    fmt: StrLit,
    /// Types after the format string
    type_tokens: Vec<Path>,
}
/// Input string, format string and types for sscanf and sscanf_unescaped
struct Scanf {
    /// input to run the sscanf on
    src_str: Expr,
    /// format string and types
    inner: ScanfInner,
}

impl Parse for ScanfInner {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(Span::call_site(), "Missing format string"));
        }

        let fmt = input.parse::<StrLit>()?;

        let type_tokens = if input.is_empty() {
            vec![]
        } else {
            input.parse::<Token![,]>()?; // the comma after the format string

            input
                .parse_terminated::<_, Token![,]>(Path::parse)?
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
            return Err(syn::Error::new(
                Span::call_site(),
                "At least 2 Parameters required: Input and format string",
            ));
        }
        let src_str = input.parse()?;
        if input.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "At least 2 Parameters required: Missing format string",
            ));
        }
        let comma = input.parse::<Token![,]>()?;
        if input.is_empty() {
            // Addition to the comment above: here we actually have a comma to point to to say:
            // "Hey, you put a comma here, put something after it". syn doesn't do this
            // because it can't rewind the input stream to check this.
            return Err(syn::Error::new_spanned(
                comma,
                "At least 2 Parameters required: Missing format string",
            ));
        }
        let inner = input.parse()?;

        Ok(Scanf { src_str, inner })
    }
}

#[proc_macro]
pub fn sscanf(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Scanf);
    sscanf_internal(input, true)
}

#[proc_macro]
pub fn sscanf_unescaped(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as Scanf);
    sscanf_internal(input, false)
}

#[proc_macro]
pub fn sscanf_get_regex(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as ScanfInner);
    let (regex, _) = match generate_regex(input, true) {
        Ok(v) => v,
        Err(e) => return e.into(),
    };
    quote!({
        #regex
        &REGEX
    })
    .into()
}

#[proc_macro_derive(FromScanf, attributes(sscanf))]
pub fn derive_from_sscanf(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    let generics = input.generics;
    let attr = derive::find_attr(input.attrs);

    let res = match input.data {
        syn::Data::Struct(data) => derive::parse_struct(ident, generics, attr, data),
        syn::Data::Enum(data) => derive::parse_enum(ident, generics, attr, data),
        syn::Data::Union(data) => derive::parse_union(ident, generics, attr, data),
    };
    match res {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn sscanf_internal(input: Scanf, escape_input: bool) -> TokenStream1 {
    let (regex, matcher) = match generate_regex(input.inner, escape_input) {
        Ok(v) => v,
        Err(e) => return e.into(),
    };
    let src_str = {
        let src_str = input.src_str;
        let (start, end) = full_span(&src_str);
        let mut param = quote_spanned!(start => &);
        param.extend(quote_spanned!(end => (#src_str)));
        // wrapping the input in a manual call to str::get ensures that the user
        // gets an appropriate error message if they try to use a non-string input
        quote!(::std::primitive::str::get(#param, ..).unwrap())
    };
    quote!(
        {
            #regex
            #[allow(clippy::needless_borrow)]
            let input: &str = #src_str;
            #[allow(clippy::needless_question_mark)]
            REGEX.captures(input)
                .ok_or_else(|| ::sscanf::Error::MatchFailed)
                .and_then(|cap| {
                    let mut src = cap.iter();
                    let src = &mut src;
                    src.next().unwrap(); // skip the full match

                    #[cfg(debug_assertions)]
                    let mut len = src.len();

                    let mut matcher = || -> ::std::result::Result<_, ::std::boxed::Box<dyn ::std::error::Error>> {
                        ::std::result::Result::Ok(( #(#matcher),* ))
                    };
                    let res = matcher().map_err(|e| ::sscanf::Error::ParsingFailed(e));

                    if src.len() != 0 {
                        panic!("{} captures generated, but {} were taken",
                            REGEX.captures_len(), REGEX.captures_len() - src.len()
                        );
                    }
                    res
                })
        }
    )
    .into()
}

fn generate_regex(
    input: ScanfInner,
    escape_input: bool,
) -> Result<(TokenStream, Vec<TokenStream>)> {
    let mut format = FormatString::new(input.fmt.to_slice(), escape_input)?;
    format.parts[0].insert(0, '^');
    format.parts.last_mut().unwrap().push('$');

    let types = input
        .type_tokens
        .into_iter()
        .map(|path| Type::Path(TypePath { qself: None, path }))
        .collect::<Vec<_>>();

    fn to_type_source<'a>(
        ph: &Placeholder<'a>,
        visited: &mut [bool],
        ph_index: &mut usize,
    ) -> Result<TypeSource<'a>> {
        let ty_source = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text().parse::<usize>() {
                if n >= visited.len() {
                    let msg = format!("type index {} out of range of {} types", n, visited.len());
                    return name.err(&msg);
                }
                visited[n] = true;
                TypeSource::External(n)
            } else {
                TypeSource::Inline {
                    ty: to_type(name)?,
                    src: name.clone(),
                }
            }
        } else {
            let n = *ph_index;
            *ph_index += 1;
            if n >= visited.len() {
                let msg = format!("more placeholders than types provided");
                return ph.src.err(&msg);
            }
            visited[n] = true;
            TypeSource::External(n)
        };
        Ok(ty_source)
    }

    let mut ph_index = 0;
    let mut visited = vec![false; types.len()];
    let mut fields = vec![];
    let mut error = Error::builder();

    for ph in &format.placeholders {
        match to_type_source(ph, &mut visited, &mut ph_index) {
            Ok(ty_source) => fields.push(Field::from_source(ty_source)),
            Err(e) => error.push(e),
        }
    }

    for (visited, ty) in visited.iter().zip(types.iter()) {
        if !*visited {
            error.with_spanned(ty, "unused type");
        }
    }

    error.ok_or_build()?;

    let ph_indices = (0..fields.len()).collect::<Vec<_>>();
    let regex_parts = RegexParts::new(&format, &ph_indices, &fields, &types)?;

    let regex = regex_parts.regex();
    let num_captures = regex_parts.num_captures();
    let regex = quote!(::sscanf::lazy_static::lazy_static! {
        static ref REGEX: ::sscanf::regex::Regex = {
            let regex_str = #regex;
            let regex = ::sscanf::regex::Regex::new(regex_str)
                .expect("sscanf: Cannot generate Regex");

            const NUM_CAPTURES: ::std::primitive::usize = #num_captures;

            if regex.captures_len() != NUM_CAPTURES {
                panic!(
                    "sscanf: Regex has {} capture groups, but {} were expected.{}",
                    regex.captures_len(), NUM_CAPTURES, #WRONG_CAPTURES_HINT
                );
            }
            regex
        };
    });

    Ok((regex, regex_parts.from_matches_builder))
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

fn to_type(src: &StrLitSlice) -> Result<Type> {
    // dirty hack #493: a type in a string needs to be converted to a Type token, because quote
    // would surround a String with '"', and we don't want that. And we can't change that
    // other than changing the type of the variable.
    // So we parse from String to TokenStream, then parse from TokenStream to Path.
    // The alternative would be to construct the Path ourselves, but path has _way_ too
    // many parts to it with variable stuff and incomplete constructors, that's too
    // much work.
    let catcher = || -> syn::Result<Type> {
        let tokens = src.text().parse::<TokenStream>()?;
        let span = src.span();
        let path = syn::parse2::<Path>(quote_spanned!(span => #tokens))?;
        // we don't parse directly to a Type to give better error messages:
        // Path says "expected identifier"
        // Type says "expected one of: `for`, parentheses, `fn`, `unsafe`, ..."
        // because syn::Type contains too many other variants.
        Ok(Type::Path(TypePath { qself: None, path }))
    };
    catcher().map_err(|err| {
        let hint =  "The syntax for placeholders is {<type>} or {<type>:<config>}. Make sure <type> is a valid type or index.";
        let hint2 = "If you want syntax highlighting and better errors, place the type in the arguments after the format string while debugging";
        src.error(
            &format!("Invalid type in placeholder: {}.\nHint: {}\n{}", err, hint, hint2),
        )
    })
}
