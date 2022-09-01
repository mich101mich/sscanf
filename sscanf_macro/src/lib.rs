//! Crate with proc_macros for [sscanf](https://crates.io/crates/sscanf). Not usable as a standalone crate.

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    DeriveInput, Expr, Path, Token,
};

mod attributes;
mod error;
mod format_option;
mod format_string;
mod placeholder;
mod str_lit;

pub(crate) use attributes::*;
pub(crate) use error::*;
pub(crate) use format_option::*;
pub(crate) use format_string::*;
pub(crate) use placeholder::*;
pub(crate) use str_lit::*;

mod derive;

/// Format string and types for scanf_get_regex. Shared by scanf and scanf_unescaped
struct ScanfInner {
    /// the format string
    fmt: StrLit,
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
        Err(e) => return e.into(),
    };
    quote!({
        #regex
        &REGEX
    })
    .into()
}

#[proc_macro_derive(FromScanf, attributes(scanf))]
pub fn derive_from_scanf(input: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(input as DeriveInput);

    let res = match input.data {
        syn::Data::Struct(data) => derive::parse_struct(input.ident, input.attrs, data),
        syn::Data::Enum(data) => derive::parse_enum(input.ident, input.attrs, data),
        syn::Data::Union(data) => derive::parse_union(input.ident, input.attrs, data),
    };
    match res {
        Ok(res) => res.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn scanf_internal(input: Scanf, escape_input: bool) -> TokenStream1 {
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
        quote!(::std::primitive::str::get(#param, ..).unwrap_or_else(|| panic!("a:{}:{}", file!(), line!())))
    };
    quote!(
        {
            #regex
            #[allow(clippy::needless_borrow)]
            let input: &str = #src_str;
            #[allow(clippy::needless_question_mark)]
            REGEX.captures(input)
                .ok_or_else::<Box<dyn ::std::error::Error>, _>(|| ::std::boxed::Box::new(::sscanf::RegexMatchFailed))
                .and_then(|cap| {
                    let mut src = cap.iter();
                    src.next().unwrap(); // skip the full match
                    let mut len = src.len();
                    let start_len = len;
                    let res = ::std::result::Result::Ok(( #(#matcher),* ));
                    if src.len() != 0 {
                        panic!("{} captures generated, but {} were taken
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid
forming a capture group like this:
    ...(...)...  =>  ...(?:...)...
",
                            start_len, len
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
    let format = FormatString::new(input.fmt.to_slice(), escape_input)?;

    let types = &input.type_tokens;

    // these need to be Vec instead of TokenStream to allow adding the comma separators later
    let mut regex_builder = vec![];
    let mut num_captures_builder = vec![quote!(1)]; // +1 for the entire match
    let mut from_matches_builder = vec![];
    let mut ph_index = 0;
    let mut visited = vec![false; types.len()];
    let mut error = Error::builder();
    // if there are n types, there are n+1 regex_parts, so add the first n during this loop and
    // add the last one afterwards
    for (prefix, ph) in format.parts.iter().zip(format.placeholders.iter()) {
        regex_builder.push(quote!(#prefix));

        let (ty, ty_src) = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text.parse::<usize>() {
                if let Some(ty) = types.get(n) {
                    visited[n] = true;
                    (ty.clone(), None)
                } else {
                    let msg = format!("type index {} out of range of {} types", n, types.len());
                    error.with_error(name.error(&msg));
                    continue;
                }
            } else {
                match to_path(name) {
                    Ok(path) => (path, Some(name)),
                    Err(e) => {
                        error.with_error(e);
                        continue;
                    }
                }
            }
        } else {
            let n = ph_index;
            ph_index += 1;
            if let Some(ty) = types.get(n) {
                visited[n] = true;
                (ty.clone(), None)
            } else {
                let msg = format!("more placeholders than types provided");
                error.with_error(ph.src.error(&msg));
                continue;
            }
        };

        let (start, end) = if let Some(src) = ty_src.as_ref() {
            let span = src.span();
            (span, span)
        } else {
            full_span(&ty)
        };

        let regex = if let Some(config) = ph.config.as_ref() {
            use FormatOptionKind::*;
            match config.kind {
                Regex(ref regex) => quote!(#regex),
                Radix(ref _radix) => quote!(".*?"),
            }
        } else {
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
        };
        regex_builder.push(regex);

        let (num_captures, converter) = if ty.to_token_stream().to_string() == "str" {
            // str is special, because the type is actually &str
            let cap = quote_spanned!(start => 1);
            let conv = quote_spanned!(start => src.next().expect("c").expect("d").as_str());
            (cap, conv)
        } else {
            let mut cap = quote_spanned!(start => <#ty as );
            cap.extend(quote_spanned!(end => ::sscanf::FromScanf>::NUM_CAPTURES));

            let mut conv = quote_spanned!(start => <#ty as );
            conv.extend(quote_spanned!(end => ::sscanf::FromScanf>::from_matches(&mut src)?));

            (cap, conv)
        };

        let from_matches = quote!({
            let value = #converter;
            let n = len - src.len();
            if n != #num_captures {
                panic!("{}::NUM_CAPTURES = {} but {} were taken
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid
forming a capture group like this:
    ...(...)...  =>  ...(?:...)...
",
                    stringify!(#ty), #num_captures, n
                );
            }
            len = src.len();
            value
        });

        num_captures_builder.push(num_captures);
        from_matches_builder.push(from_matches);
    }

    if !error.is_empty() {
        return error.build_err();
    }

    // add the last regex_part
    {
        let suffix = format
            .parts
            .last()
            .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()));
        regex_builder.push(quote!(#suffix));
    }

    error = Error::builder();
    for (visited, ty) in visited.iter().zip(types.iter()) {
        if !*visited {
            error.with_spanned(ty, "unused type");
        }
    }
    if !error.is_empty() {
        return error.build_err();
    }

    let regex = quote!(::sscanf::lazy_static::lazy_static! {
        static ref REGEX: ::sscanf::regex::Regex = {
            let regex_str = ::sscanf::const_format::concatcp!( #(#regex_builder),* );
            let regex = ::sscanf::regex::Regex::new(regex_str)
                .expect("scanf: Cannot generate Regex");

            const NUM_CAPTURES: usize = #(#num_captures_builder)+*;

            if regex.captures_len() != NUM_CAPTURES {
                panic!("scanf: Regex has {} capture groups, but {} were expected.
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid
forming a capture group like this:
    ...(...)...  =>  ...(?:...)...
", regex.captures_len(), NUM_CAPTURES);
            }
            regex
        };
    });

    Ok((regex, from_matches_builder))
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

fn to_path(src: &StrLitSlice) -> Result<Path> {
    // dirty hack #493: type_name needs to be converted to a Path token, because quote
    // would surround a String with '"', and we don't want that. And we can't change that
    // other than changing the type of the variable.
    // So we parse from String to TokenStream, then parse from TokenStream to Path.
    // The alternative would be to construct the Path ourselves, but path has _way_ too
    // many parts to it with variable stuff and incomplete constructors, that's too
    // much work.
    src.text
        .parse::<TokenStream>()
        .map_err(|err| err.to_string())
        .and_then(|tokens| {
            syn::parse2::<Path>(quote!(#tokens))
            .map_err(|err| err.to_string())
        })
        .map_err(|err| {
            let hint =  "The syntax for placeholders is {<type>} or {<type>:<config>}. Make sure <type> is a valid type.";
            let hint2 = "If you want syntax highlighting and better errors, place the type in the arguments after the format string while debugging";
            src.error(
                &format!("Invalid type in placeholder: {}.\nHint: {}\n{}", err, hint, hint2),
            )
        })
}
