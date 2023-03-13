use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

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
    CowStr,
    FromType(syn::Type, FullSpan),
    Custom(TokenStream),
}

impl ToTokens for Converter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Converter::Str => tokens.extend(quote! {
                src.next()
                   .expect(::sscanf::errors::EXPECT_NEXT_HINT)
                   .expect(::sscanf::errors::EXPECT_CAPTURE_HINT)
                   .as_str()
            }),
            Converter::CowStr => tokens.extend(quote! {
                ::std::borrow::Cow::Borrowed(
                    src.next()
                        .expect(::sscanf::errors::EXPECT_NEXT_HINT)
                        .expect(::sscanf::errors::EXPECT_CAPTURE_HINT)
                        .as_str()
                )
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
                });
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
                            stringify!(#ty), expected, n, ::sscanf::errors::WRONG_CAPTURES_HINT
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

    pub fn new(format: &FormatString, type_sources: &[Type]) -> Result<Self> {
        let mut ret = Self::empty();

        // if there are n types, there are n+1 regex_parts, so add the first n during this loop and
        // add the last one afterwards
        for ((part, ph), ty) in format
            .parts
            .iter()
            .zip(format.placeholders.iter())
            .zip(type_sources)
        {
            ret.push_literal(part);

            let inner = ty.inner();
            let span = ty.full_span();

            let mut converter = None;

            let regex = if let Some(config) = ph.config.as_ref() {
                use FormatOptionKind::*;
                match &config.kind {
                    Regex(regex) => RegexPart::Custom(regex.clone()),
                    Radix { radix, prefix } => {
                        let (regex, conv) = regex_from_radix(*radix, *prefix, ty)?;
                        converter = Some(conv);
                        regex
                    }
                    Hashtag => {
                        return config.src.err("unsupported use of '#'");
                    }
                }
            } else {
                match ty.kind {
                    TypeKind::Str(_) | TypeKind::CowStr(_) => {
                        let token = quote! { str }.with_span(inner.span());
                        let ty = syn::parse2(token).unwrap();
                        RegexPart::FromType(ty, span)
                    }
                    _ => RegexPart::FromType(inner.clone(), span),
                }
            };
            ret.regex_builder.push(regex);

            let (num_captures, converter) = match ty.kind {
                TypeKind::Str(_) => (NumCaptures::One, Converter::Str),
                TypeKind::CowStr(_) => (NumCaptures::One, Converter::CowStr),
                TypeKind::Other => (
                    NumCaptures::FromType(inner.clone(), span),
                    converter.unwrap_or_else(|| Converter::FromType(inner.clone(), span)),
                ),
            };

            ret.matchers.push(Matcher {
                ty: inner.clone(),
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
        quote! { ::sscanf::const_format::concatcp!( #(#regex_builder),* ) }
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
    ty: &Type,
) -> Result<(RegexPart, Converter)> {
    let ty_string = ty.to_token_stream().to_string();

    let num_digits_binary = binary_length(&ty_string).ok_or_else(|| {
        let msg = "radix options only work on primitive numbers from std with no path or alias";
        ty.error(msg) // checked in tests/fail/<channel>/invalid_radix_option.rs
    })?;

    let signed = ty_string.starts_with('i');
    let sign = if signed { "[-+]?" } else { "\\+?" };

    let prefix_string = match prefix_policy {
        PrefixPolicy::Optional(prefix) => format!("(?:{})?", prefix),
        PrefixPolicy::Forced(prefix) => prefix.to_string(),
        PrefixPolicy::Never => String::new(),
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

    let num_digits = if radix == 2 {
        num_digits_binary
    } else {
        // digit conversion:   num_digits_in_base_a = num_digits_in_base_b * log(b) / log(a)
        // where log can be any type of logarithm. Since binary is base 2 and log_2(2) = 1,
        // we can use log_2 to simplify the math
        f32::ceil(num_digits_binary as f32 / f32::log2(radix as f32)) as u32
    };

    let regex = format!(
        "(?i:{sign}{prefix}[{digits}]{{1,{n}}})",
        sign = sign,
        prefix = prefix_string,
        digits = possible_chars,
        n = num_digits
    );

    // we know ty is a primitive type without path, which are always just one token
    // => no Span voodoo necessary
    let ty = ty.inner();
    let span = ty.span();

    let get_input = quote! { src.next()
        .expect(::sscanf::errors::EXPECT_NEXT_HINT)
        .expect(::sscanf::errors::EXPECT_CAPTURE_HINT)
        .as_str()
    };

    fn create_converter(
        ty: &syn::Type,
        radix: u32,
        signed: bool,
        prefix_policy: PrefixPolicy,
        get_input: TokenStream,
    ) -> TokenStream {
        let (prefix, no_prefix_handler) = match prefix_policy {
            PrefixPolicy::Never => {
                return quote! {{
                    let input = #get_input;
                    #ty::from_str_radix(input, #radix)?
                }};
            }
            PrefixPolicy::Optional(prefix) => (prefix, quote! { /* do nothing */ }),
            PrefixPolicy::Forced(prefix) => (
                prefix,
                quote! {
                    ::std::option::Option::None.ok_or(::sscanf::errors::MissingPrefixError::#prefix)?;
                    #[allow(unreachable_code)]
                },
            ),
        };
        let prefix_lowercase = prefix.to_string();
        let prefix_uppercase = prefix_lowercase.to_uppercase();
        let prefix_matcher = quote! {
            no_sign.strip_prefix(#prefix_lowercase).or_else(|| no_sign.strip_prefix(#prefix_uppercase))
        };

        if signed {
            quote! {{
                let input = #get_input;
                let (negative, no_sign) = match input.strip_prefix('-') {
                    ::std::option::Option::Some(no_sign) => (true, no_sign),
                    ::std::option::Option::None => (false, input.strip_prefix('+').unwrap_or(input)),
                };
                if let ::std::option::Option::Some(no_sign_prefix) = #prefix_matcher {
                    if negative {
                        // re-package `no_sign_prefix` into a string that includes the sign, because otherwise
                        // it might cause faulty overflow errors on numbers like -128i8
                        let input = ::std::format!("-{}", no_sign_prefix);
                        #ty::from_str_radix(&input, #radix)?
                    } else {
                        #ty::from_str_radix(no_sign_prefix, #radix)?
                    }
                } else {
                    #no_prefix_handler
                    #ty::from_str_radix(input, #radix)? // note the use of `input` here to include the sign
                }
            }}
        } else {
            quote! {{
                let input = #get_input;
                let no_sign = input.strip_prefix('+').unwrap_or(input);
                if let ::std::option::Option::Some(no_sign_prefix) = #prefix_matcher {
                    #ty::from_str_radix(no_sign_prefix, #radix)?
                } else {
                    #no_prefix_handler
                    #ty::from_str_radix(no_sign, #radix)?
                }
            }}
        }
    }

    let converter = create_converter(ty, radix as u32, signed, prefix_policy, get_input);

    Ok((
        RegexPart::Custom(regex),
        Converter::Custom(converter.with_span(span)),
    ))
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
