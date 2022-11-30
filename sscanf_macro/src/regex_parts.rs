use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

/// A workaround for Spans on stable Rust.
///
/// Span manipulation doesn't work on stable Rust, which also means that spans cannot be joined
/// together. This means that any compiler errors that occur would only point at the first token
/// of the spanned expression, which is not very helpful.
///
/// The workaround, as demonstrated by `syn::Error::new_spanned`, is to have the first part of the
/// spanned expression be spanned with the first part of the source span, and the second part of the
/// spanned expression be spanned with the second part of the source span. The compiler only looks
/// at the start and end of the span and underlines everything in between, so this works.
#[derive(Copy, Clone)]
pub struct FullSpan(Span, Span);

impl FullSpan {
    pub fn from_spanned<T: ToTokens + syn::spanned::Spanned>(span: &T) -> Self {
        let start = span.span();
        let end = span
            .to_token_stream()
            .into_iter()
            .last()
            .map(|t| t.span())
            .unwrap_or(start);
        Self(start, end)
    }
    pub fn apply(&self, mut a: TokenStream, mut b: TokenStream) -> TokenStream {
        a.set_span(self.0);
        b.set_span(self.1);
        quote! { #a #b }
    }
}

pub struct TypeSource<'a> {
    pub ty: syn::Type,
    pub source: Option<StrLitSlice<'a>>,
}

#[allow(unused)]
impl<'a> TypeSource<'a> {
    pub fn full_span(&self) -> FullSpan {
        if let Some(source) = self.source.as_ref() {
            let span = source.span();
            FullSpan(span, span)
        } else {
            FullSpan::from_spanned(&self.ty)
        }
    }
    pub fn err<T>(&self, message: &str) -> Result<T> {
        Err(self.error(message))
    }
    pub fn error(&self, message: &str) -> Error {
        if let Some(source) = self.source.as_ref() {
            source.error(message)
        } else {
            Error::new_spanned(&self.ty, message)
        }
    }
}

#[derive(Clone)]
pub enum NumCaptures {
    One,
    FromType(syn::Type, FullSpan),
}

impl ToTokens for NumCaptures {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NumCaptures::One => tokens.extend(quote! { 1 }),
            NumCaptures::FromType(ty, span) => tokens.extend(span.apply(
                quote! { <#ty as },
                quote! { ::sscanf::FromScanf>::NUM_CAPTURES },
            )),
        }
    }
}

pub enum RegexPart {
    Literal(String),
    FromType(syn::Type, FullSpan),
    Custom(String),
}

impl ToTokens for RegexPart {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RegexPart::Literal(literal) => tokens.extend(quote! { #literal }),
            RegexPart::FromType(ty, span) => {
                // proc_macros don't have any type information, so we cannot check if the type
                // implements the trait, so we wrap it in this verbose <#ty as Trait> code,
                // so that the compiler can check if the trait is implemented, and, most importantly,
                // tell the user if they forgot to implement the trait.
                // The code is split into two parts in case the type consists of more
                // than one token (like `std::vec::Vec`), so that the FullSpan workaround can be
                // applied.
                tokens.extend(span.apply(
                    quote! { <#ty as },
                    quote! { ::sscanf::RegexRepresentation>::REGEX },
                ));
            }
            RegexPart::Custom(custom) => tokens.extend(quote! { #custom }),
        }
    }
}

pub enum Converter {
    Str,
    FromType(syn::Type, FullSpan),
    Custom(TokenStream),
}

impl ToTokens for Converter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Converter::Str => tokens.extend(quote! {
                src.next()
                   .expect(::sscanf::EXPECT_NEXT_HINT)
                   .expect(::sscanf::EXPECT_CAPTURE_HINT)
                   .as_str()
            }),
            Converter::FromType(ty, span) => {
                let call = span.apply(
                    quote! { ::sscanf::FromScanf },
                    quote! { ::from_matches(&mut *src) },
                );
                tokens.extend(quote! {
                    {
                        let value: #ty = #call?;
                        value
                    }
                })
            }
            Converter::Custom(custom) => tokens.extend(custom.clone()),
        }
    }
}

pub struct Matcher {
    pub ty: syn::Type,
    pub num_captures: NumCaptures,
    pub converter: Converter,
}

impl ToTokens for Matcher {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = &self.ty;
        let num_captures = &self.num_captures;
        let converter = &self.converter;
        tokens.extend(quote! {
            {
                #[cfg(debug_assertions)]
                let start_len = src.len();

                let value = #converter;

                #[cfg(debug_assertions)]
                {
                    let n = start_len - src.len();
                    let expected = #num_captures;
                    if n != expected {
                        panic!(
                            "sscanf: {}::NUM_CAPTURES = {} but {} were taken{}",
                            stringify!(#ty), expected, n, ::sscanf::WRONG_CAPTURES_HINT
                        );
                    }
                }
                value
            }
        });
    }
}

pub struct RegexParts {
    pub regex_builder: Vec<RegexPart>,
    pub matchers: Vec<Matcher>,
}

impl RegexParts {
    pub fn empty() -> Self {
        Self {
            regex_builder: vec![],
            matchers: vec![],
        }
    }

    pub fn push_literal(&mut self, literal: impl Into<String>) {
        self.regex_builder.push(RegexPart::Literal(literal.into()));
    }

    pub fn new(format: &FormatString, type_sources: &[TypeSource]) -> Result<Self> {
        let mut ret = Self::empty();

        // if there are n types, there are n+1 regex_parts, so add the first n during this loop and
        // add the last one afterwards
        for ((part, ph), ty_source) in format
            .parts
            .iter()
            .zip(format.placeholders.iter())
            .zip(type_sources)
        {
            ret.push_literal(part);

            let ty = &ty_source.ty;
            let span = ty_source.full_span();

            let mut converter = None;

            let regex = if let Some(config) = ph.config.as_ref() {
                use FormatOptionKind::*;
                match &config.kind {
                    Regex(regex) => RegexPart::Custom(regex.clone()),
                    Radix { radix, prefix } => {
                        let (regex, conv) = regex_from_radix(*radix, *prefix, ty_source)?;
                        converter = Some(conv);
                        regex
                    }
                    Hashtag => {
                        return config.src.err("unsupported use of '#'");
                    }
                }
            } else {
                RegexPart::FromType(ty.clone(), span)
            };
            ret.regex_builder.push(regex);

            let (num_captures, converter) = if ty.to_token_stream().to_string() == "str" {
                // str is special, because the type is actually &str
                (NumCaptures::One, converter.unwrap_or(Converter::Str))
            } else {
                (
                    NumCaptures::FromType(ty.clone(), span),
                    converter.unwrap_or_else(|| Converter::FromType(ty.clone(), span)),
                )
            };

            ret.matchers.push(Matcher {
                ty: ty.clone(),
                num_captures,
                converter,
            });
        }

        // add the last regex_part
        {
            let suffix = format.parts.last().unwrap();
            ret.push_literal(suffix);
        }

        Ok(ret)
    }

    pub fn regex(&self) -> TokenStream {
        let regex_builder = &self.regex_builder;
        quote!(::sscanf::const_format::concatcp!( #(#regex_builder),* ))
    }
    pub fn num_captures_list(&self) -> Vec<NumCaptures> {
        let mut num_captures = vec![NumCaptures::One]; // for the whole match
        for matcher in &self.matchers {
            num_captures.push(matcher.num_captures.clone());
        }
        num_captures
    }
    pub fn num_captures(&self) -> TokenStream {
        let num_captures = self.num_captures_list();
        quote! { #(#num_captures)+* }
    }
}

fn regex_from_radix(
    radix: u8,
    prefix_policy: PrefixPolicy,
    ty_source: &TypeSource,
) -> Result<(RegexPart, Converter)> {
    let ty_string = ty_source.ty.to_token_stream().to_string();

    let num_digits_binary = binary_length(&ty_string).ok_or_else(|| {
        let msg = "radix options only work on primitive numbers from std with no path or alias";
        ty_source.error(msg) // checked in tests/fail/<channel>/invalid_radix_option.rs
    })?;

    let signed = ty_string.starts_with('i');
    let sign = if signed { "[-+]?" } else { "\\+?" };

    let prefix = match radix {
        2 => Some("0b"),
        8 => Some("0o"),
        16 => Some("0x"),
        _ => None,
    };
    let prefix_string = match (prefix_policy, prefix) {
        (PrefixPolicy::Optional, Some(prefix)) => format!("(?:{})?", prefix),
        (PrefixPolicy::Forced, Some(prefix)) => prefix.to_owned(),
        (PrefixPolicy::Never, _) => String::new(),
        _ => panic!("Invalid internal prefix configuration"),
    };

    // possible characters for digits
    use std::cmp::Ordering::*;
    let possible_chars = match radix.cmp(&10) {
        Less => format!("0-{}", radix - 1),
        Equal => "0-9a".to_string(),
        Greater => {
            let last_letter = (b'a' + radix - 10) as char;
            format!("0-9a-{}", last_letter)
        }
    };

    // digit conversion:   num_digits_in_base_a = num_digits_in_base_b / log(b) * log(a)
    // where log can be any type of logarithm. Since binary is base 2 and log_2(2) = 1,
    // we can use log_2 to simplify the math
    let num_digits = f32::ceil(num_digits_binary as f32 / f32::log2(radix as f32)) as u8;

    let regex = format!(
        "(?i:{sign}{prefix}[{digits}]{{1,{n}}})",
        sign = sign,
        prefix = prefix_string,
        digits = possible_chars,
        n = num_digits
    );

    // we know ty is a primitive type without path, which are always just one token
    // => no Span voodoo necessary
    let span = ty_source.full_span().0;
    let ty = &ty_source.ty;

    let get_input = quote!(src
        .next()
        .expect(::sscanf::EXPECT_NEXT_HINT)
        .expect(::sscanf::EXPECT_CAPTURE_HINT)
        .as_str());

    let radix = radix as u32;
    let mut converter = if prefix_policy == PrefixPolicy::Never {
        quote!({
            let input = #get_input;
            #ty::from_str_radix(input, #radix)?
        })
    } else {
        let no_prefix_handler = match prefix_policy {
            PrefixPolicy::Optional => quote!(unwrap_or(s)),
            PrefixPolicy::Forced => quote!(ok_or(::sscanf::MissingPrefixError)?),
            PrefixPolicy::Never => unreachable!(),
        };
        let prefix_lowercase = prefix.expect("Invalid internal prefix configuration");
        let prefix_uppercase = prefix_lowercase.to_uppercase();
        let prefix_matcher = quote!(
            s.strip_prefix(#prefix_lowercase).or_else(|| s.strip_prefix(#prefix_uppercase))
        );

        if signed {
            quote!({
                let input = #get_input;
                let (negative, s) = match input.strip_prefix('-') {
                    Some(s) => (true, s),
                    None => (false, input.strip_prefix('+').unwrap_or(input)),
                };
                let s = #prefix_matcher.#no_prefix_handler;
                #ty::from_str_radix(s, #radix).map(|i| if negative { -i } else { i })?
            })
        } else {
            quote!({
                let input = #get_input;
                let s = input.strip_prefix('+').unwrap_or(input);
                let s = #prefix_matcher.#no_prefix_handler;
                #ty::from_str_radix(s, #radix)?
            })
        }
    };

    converter.set_span(span);

    Ok((RegexPart::Custom(regex), Converter::Custom(converter)))
}

fn binary_length(ty: &str) -> Option<u32> {
    match ty {
        "u8" | "i8" => Some(u8::BITS),
        "u16" | "i16" => Some(u16::BITS),
        "u32" | "i32" => Some(u32::BITS),
        "u64" | "i64" => Some(u64::BITS),
        "u128" | "i128" => Some(u128::BITS),
        "usize" | "isize" => Some(usize::BITS),
        _ => None,
    }
}
