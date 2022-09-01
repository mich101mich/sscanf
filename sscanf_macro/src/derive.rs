use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{parse2, spanned::Spanned, Attribute, DataEnum, DataStruct, DataUnion, Ident, Type};

use crate::*;

struct Field {
    ident: Ident,
    ty: Type,
    default: Option<TokenStream>,
}

pub fn parse_struct(name: Ident, attrs: Vec<Attribute>, data: DataStruct) -> Result<TokenStream> {
    let attr = if let Some(attr) = attrs.iter().find(|a| a.path.is_ident("scanf")) {
        attr
    } else {
        let msg = "structs must have a #[scanf(format=\"...\")] attribute";
        return Error::err(Span::call_site(), msg);
    };

    let mut configs = FormatAttribute::from_attrs(attr)?;
    let format_src = if let Some(format) = configs.remove("format") {
        format
    } else {
        match configs.len() {
            0 => {
                let msg = "structs require a format=\"...\" option";
                return Error::err(Span::call_site(), msg);
            }
            1 => {
                let (name, elem) = configs
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()));
                let msg = format!("unexpected option: `{}`. Expected `format`", name);
                return Error::err_spanned(elem.name, msg);
            }
            _ => {
                let mut error = Error::builder();
                for (name, option) in configs {
                    error.with_spanned(option.src, format!("unexpected option: `{}`", name));
                }
                return error.build_err();
            }
        }
    };

    if !configs.is_empty() {
        return Error::err_spanned(attr, "structs may only have a single `format` option");
    }

    let format = FormatString::new(format_src.value.to_slice(), true)?;

    let mut fields = vec![];
    let mut field_map = HashMap::new();
    for (i, field) in data.fields.into_iter().enumerate() {
        let ty = field.ty;
        let ident = field
            .ident
            .unwrap_or_else(|| Ident::new(&format!("{}", i), ty.span()));

        field_map.insert(ident.to_string(), i);

        let mut default = None;

        if let Some(attr) = field.attrs.into_iter().find(|a| a.path.is_ident("scanf")) {
            if !attr.tokens.is_empty() {
                default = Some(parse2::<DefaultAttribute>(attr.tokens)?.0);
            }
        };

        fields.push(Field {
            ident,
            ty: ty,
            default,
        });
    }

    let mut regex_builder = vec![];
    let mut num_captures_builder = vec![quote!(0)]; // +0 in case of unit structs
    let mut from_matches_builder = vec![];
    let mut ph_index = 0;
    let mut visited = vec![false; fields.len()];
    let mut error = Error::builder();
    for (prefix, ph) in format.parts.iter().zip(format.placeholders.iter()) {
        regex_builder.push(quote!(#prefix));

        let index = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text.parse::<usize>() {
                if n < fields.len() {
                    n
                } else {
                    let msg = format!("field index {} out of range of {} fields", n, fields.len());
                    error.with_error(name.error(&msg));
                    continue;
                }
            } else if let Some(i) = field_map.get(name.text) {
                *i
            } else {
                error.with_error(name.error(&format!("field `{}` not found", name.text)));
                continue;
            }
        } else {
            ph_index += 1;
            ph_index - 1
        };

        if visited[index] {
            let msg = format!(
                "field `{}` specified more than once",
                fields[index].ident.to_string()
            );
            error.with_error(ph.src.error(&msg));
            continue;
        }
        visited[index] = true;

        let ty = &fields[index].ty;

        let (start, end) = full_span(&ty);

        let regex = if let Some(config) = ph.config.as_ref() {
            use FormatOptionKind::*;
            match config.kind {
                Regex(ref regex) => quote!(#regex),
                Radix(ref _radix) => quote!(".*?"),
            }
        } else {
            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::sscanf::RegexRepresentation>::REGEX));
            s
        };
        regex_builder.push(regex);

        let num_captures = {
            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::sscanf::FromScanf>::NUM_CAPTURES));
            s
        };

        let converter = {
            let mut s = quote_spanned!(start => <#ty as );
            s.extend(quote_spanned!(end => ::sscanf::FromScanf>::from_matches(&mut src)?));
            s
        };
        let ident = &fields[index].ident;
        let from_matches = quote!(#ident: {
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

    {
        let suffix = format
            .parts
            .last()
            .unwrap_or_else(|| panic!("a:{}:{}", file!(), line!()));
        regex_builder.push(quote!(#suffix));
    }

    error = Error::builder();
    for (visited, field) in visited.iter().zip(fields.into_iter()) {
        if *visited {
            continue;
        }
        if let Some(default) = field.default.as_ref() {
            let ident = &field.ident;
            from_matches_builder.push(quote!(#ident: #default));
        } else {
            let msg = format!("field `{}` has no format or default value", field.ident);
            error.with_spanned(field.ident, msg);
        }
    }
    if !error.is_empty() {
        return error.build_err();
    }

    let regex_impl = quote!(
        impl ::sscanf::RegexRepresentation for #name {
            const REGEX: &'static str = ::sscanf::const_format::concatcp!( #(#regex_builder),* );
        }
    );

    let from_scanf_impl = quote!(
        impl ::sscanf::FromScanf for #name {
            type Err = FromScanfFailedError;
            const NUM_CAPTURES: usize = #(#num_captures_builder)+*;
            fn from_matches(src: &mut ::sscanf::regex::SubCaptureMatches) -> ::std::result::Result<Self, Self::Err> {
                let mut len = src.len();
                let start_len = len;
                let catcher = || -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
                    Ok(#name {
                        #(#from_matches_builder),*
                    })
                };
                let res = catcher().map_err(|error| FromScanfFailedError {
                    type_name: stringify!(#name),
                    error,
                });
                let n = src.len() - start_len;
                if n != Self::NUM_CAPTURES {
                    panic!("{}::NUM_CAPTURES = {} but {} were taken
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid
forming a capture group like this:
    ...(...)...  =>  ...(?:...)...
",
                        stringify!(#name), Self::NUM_CAPTURES, n
                    );
                }
                Ok(res)
            }
        }
    );

    Ok(quote!(
        #regex_impl
        #from_scanf_impl
    ))
}

pub fn parse_enum(name: Ident, attrs: Vec<Attribute>, data: DataEnum) -> Result<TokenStream> {
    return Err(Error::new_spanned(name, "todo"));
}

pub fn parse_union(name: Ident, attrs: Vec<Attribute>, data: DataUnion) -> Result<TokenStream> {
    return Err(Error::new_spanned(name, "union is yet not supported"));
}
