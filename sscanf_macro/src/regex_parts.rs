use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Ident, Type};

use crate::*;

pub const WRONG_CAPTURES_HINT: &str = r#"
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  "
"#;

pub struct Field<'a> {
    pub ident: FieldIdent,
    pub ty_source: TypeSource<'a>,
}

pub enum FieldIdent {
    Named(Ident),
    Index(Literal),
    None,
}
impl FieldIdent {
    pub fn from_index(i: usize, span: Span) -> FieldIdent {
        let mut literal = Literal::usize_unsuffixed(i);
        literal.set_span(span);
        FieldIdent::Index(literal)
    }
}
impl ToTokens for FieldIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldIdent::Named(ident) => tokens.extend(quote!(#ident:)),
            FieldIdent::Index(index) => tokens.extend(quote!(#index:)),
            FieldIdent::None => {}
        }
    }
}
impl std::fmt::Display for FieldIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldIdent::Named(ident) => write!(f, "{}", ident),
            FieldIdent::Index(index) => write!(f, "{}", index),
            FieldIdent::None => write!(f, "_"),
        }
    }
}

pub enum TypeSource<'a> {
    Inline { ty: Type, src: StrLitSlice<'a> },
    External(usize),
}
#[allow(unused)]
impl<'a> TypeSource<'a> {
    pub fn ty<'b>(&'b self, types: &'b [Type]) -> &'b Type {
        match self {
            TypeSource::Inline { ty, .. } => ty,
            TypeSource::External(index) => &types[*index],
        }
    }
    pub fn span(&self, types: &[Type]) -> Span {
        match self {
            TypeSource::Inline { src, .. } => src.span(),
            TypeSource::External(index) => types[*index].span(),
        }
    }
    pub fn full_span(&self, types: &[Type]) -> (Span, Span) {
        match self {
            TypeSource::Inline { src, .. } => {
                let span = src.span();
                (span, span)
            }
            TypeSource::External(index) => full_span(&types[*index]),
        }
    }
    pub fn err<T>(&self, types: &[Type], message: &str) -> Result<T> {
        Err(self.error(types, message))
    }
    pub fn error(&self, types: &[Type], message: &str) -> Error {
        match self {
            TypeSource::Inline { src, .. } => src.error(message),
            TypeSource::External(index) => Error::new_spanned(&types[*index], message),
        }
    }
}

pub struct RegexParts {
    pub regex_builder: Vec<TokenStream>,
    pub num_captures_builder: Vec<TokenStream>,
    pub from_matches_builder: Vec<TokenStream>,
}

impl RegexParts {
    pub fn empty() -> Self {
        Self {
            regex_builder: vec![],
            num_captures_builder: vec![],
            from_matches_builder: vec![],
        }
    }

    pub fn new(
        format: &FormatString,
        ph_indices: &[usize],
        fields: &[Field],
        types: &[Type],
        handle_str: bool,
    ) -> Result<Self> {
        let mut ret = RegexParts {
            regex_builder: vec![],
            num_captures_builder: vec![],
            from_matches_builder: vec![],
        };
        ret.num_captures_builder.push(quote!(1)); // +1 for the whole match

        let mut error = Error::builder();

        // if there are n types, there are n+1 regex_parts, so add the first n during this loop and
        // add the last one afterwards
        for ((prefix, ph), index) in format
            .parts
            .iter()
            .zip(format.placeholders.iter())
            .zip(ph_indices)
        {
            ret.regex_builder.push(quote!(#prefix));

            let field = &fields[*index];
            let ty = field.ty_source.ty(types);
            let ty_string = ty.to_token_stream().to_string();
            let (start, end) = field.ty_source.full_span(types);
            let ident = &field.ident;

            let mut converter = None;

            let regex = if let Some(config) = ph.config.as_ref() {
                use FormatOptionKind::*;
                match &config.kind {
                    Regex(regex) => quote!(#regex),
                    Radix { radix, prefix } => {
                        let (regex, conv) = regex_from_radix(
                            *radix,
                            *prefix,
                            ty,
                            &field.ty_source,
                            &ty_string,
                            types,
                        )?;
                        converter = Some(conv);
                        regex
                    }
                    Hashtag => {
                        return config.src.err("unsupported use of '#'");
                    }
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
            ret.regex_builder.push(regex);

            let (num_captures, converter) = if handle_str && ty_string == "str" {
                // str is special, because the type is actually &str
                let cap = quote_spanned!(start => 1);
                let conv = quote_spanned!(start =>
                    src.next()
                       .expect(::sscanf::EXPECT_NEXT_HINT)
                       .expect(::sscanf::EXPECT_CAPTURE_HINT)
                       .as_str());
                (cap, conv)
            } else {
                let mut cap = quote_spanned!(start => <#ty as );
                cap.extend(quote_spanned!(end => ::sscanf::FromScanf>::NUM_CAPTURES));

                let conv = converter.unwrap_or_else(|| {
                    let mut conv = quote_spanned!(start => ::sscanf::FromScanf);
                    conv.extend(quote_spanned!(end => ::from_matches(&mut *src)));
                    quote!({
                        let value: #ty = #conv?;
                        value
                    })
                });

                (cap, conv)
            };

            let from_matches = quote!(#ident {
                let value = #converter;
                let n = len - src.len();
                let expected = #num_captures;
                if n != expected {
                    panic!(
                        "{}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#ty), expected, n, #WRONG_CAPTURES_HINT
                    );
                }
                len = src.len();
                value
            });

            ret.num_captures_builder.push(num_captures);
            ret.from_matches_builder.push(from_matches);
        }

        error.ok_or_build()?;

        // add the last regex_part
        {
            let suffix = format.parts.last().unwrap();
            ret.regex_builder.push(quote!(#suffix));
        }

        Ok(ret)
    }

    pub fn regex(&self) -> TokenStream {
        let regex_builder = &self.regex_builder;
        quote!(::sscanf::const_format::concatcp!( #(#regex_builder),* ))
    }
    pub fn num_captures(&self) -> TokenStream {
        let num_captures_builder = &self.num_captures_builder;
        quote!(#(#num_captures_builder)+*)
    }
    pub fn from_matches(&self) -> TokenStream {
        let from_matches_builder = &self.from_matches_builder;
        quote!({
            #(#from_matches_builder),*
        })
    }
}

fn regex_from_radix(
    radix: u8,
    prefix_policy: PrefixPolicy,
    ty: &Type,
    ty_source: &TypeSource,
    ty_string: &str,
    types: &[Type],
) -> Result<(TokenStream, TokenStream)> {
    let num_digits_binary = binary_length(&ty_string).ok_or_else(|| {
        let msg = "Radix options only work on primitive numbers from std with no path or alias";
        ty_source.error(types, msg)
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
        (PrefixPolicy::Never, None) => String::new(),
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
    let span = ty_source.span(types);

    let get_input = quote!(src
        .next()
        .expect(::sscanf::EXPECT_NEXT_HINT)
        .expect(::sscanf::EXPECT_CAPTURE_HINT)
        .as_str());

    let radix = radix as u32;
    let converter = if prefix_policy == PrefixPolicy::Never {
        quote_spanned!(span => {
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
            quote_spanned!(span => {
                let input = #get_input;
                let (negative, s) = match input.strip_prefix('-') {
                    Some(s) => (true, s),
                    None => (false, input.strip_prefix('+').unwrap_or(input)),
                };
                let s = #prefix_matcher.#no_prefix_handler;
                #ty::from_str_radix(s, #radix).map(|i| if negative { -i } else { i })?
            })
        } else {
            quote_spanned!(span => {
                let input = #get_input;
                let s = input.strip_prefix('+').unwrap_or(input);
                let s = #prefix_matcher.#no_prefix_handler;
                #ty::from_str_radix(s, #radix)?
            })
        }
    };

    Ok((quote!(#regex), converter))
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
