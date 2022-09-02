use proc_macro2::{Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Ident, Type};

use crate::*;

pub const WRONG_CAPTURES_HINT: &str = "
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid
forming a capture group like this:
    ...(...)...  =>  ...(?:...)...
";

pub struct Field<'a> {
    pub ident: FieldIdent,
    pub ty_source: TypeSource<'a>,
}

pub enum FieldIdent {
    Named(Ident),
    Index(Literal),
    None,
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

pub struct RegexParts {
    pub regex_builder: Vec<TokenStream>,
    pub num_captures_builder: Vec<TokenStream>,
    pub from_matches_builder: Vec<TokenStream>,
}

impl RegexParts {
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
            let (ty, (start, end)) = match &field.ty_source {
                TypeSource::Inline { ty, src } => {
                    let span = src.span();
                    (ty, (span, span))
                }
                TypeSource::External(i) => (&types[*i], full_span(&types[*i])),
            };
            let ident = &field.ident;

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
            ret.regex_builder.push(regex);

            let (num_captures, converter) = if handle_str
                && ty.to_token_stream().to_string() == "str"
            {
                // str is special, because the type is actually &str
                let cap = quote_spanned!(start => 1);
                let conv = quote_spanned!(start => src.next().expect("c").expect("d").as_str());
                (cap, conv)
            } else {
                let mut cap = quote_spanned!(start => <#ty as );
                cap.extend(quote_spanned!(end => ::sscanf::FromScanf>::NUM_CAPTURES));

                let mut conv = quote_spanned!(start => <#ty as );
                conv.extend(quote_spanned!(end => ::sscanf::FromScanf>::from_matches(&mut *src)?));

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

        if !error.is_empty() {
            return error.build_err();
        }

        // add the last regex_part
        {
            let suffix = format
                .parts
                .last()
                .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()));
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
