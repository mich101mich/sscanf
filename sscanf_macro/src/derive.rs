use std::collections::HashMap;

use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::{parse2, Attribute, DataEnum, DataStruct, DataUnion, Ident};

use crate::*;

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
                    .unwrap();
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
    let mut defaults = vec![];
    let mut types = vec![];
    for (i, field) in data.fields.into_iter().enumerate() {
        let ty = field.ty;
        let ident = field.ident.map(FieldIdent::Named).unwrap_or_else(|| {
            let mut literal = Literal::usize_unsuffixed(i);
            literal.set_span(ty.span());
            FieldIdent::Index(literal)
        });

        field_map.insert(ident.to_string(), i);

        let mut default = None;

        if let Some(attr) = field.attrs.into_iter().find(|a| a.path.is_ident("scanf")) {
            if !attr.tokens.is_empty() {
                default = Some(parse2::<DefaultAttribute>(attr.tokens)?.0);
            }
        };

        types.push(ty);
        fields.push(Field {
            ident,
            ty_source: TypeSource::External(i),
        });
        defaults.push(default);
    }

    let mut ph_indices = vec![];
    let mut ph_index = 0;
    let mut visited = vec![false; fields.len()];
    let mut error = Error::builder();
    for ph in &format.placeholders {
        let index = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text().parse::<usize>() {
                if n < fields.len() {
                    n
                } else {
                    let msg = format!("field index {} out of range of {} fields", n, fields.len());
                    error.with_error(name.error(&msg));
                    continue;
                }
            } else if let Some(i) = field_map.get(name.text()) {
                *i
            } else {
                error.with_error(name.error(&format!("field `{}` not found", name.text())));
                continue;
            }
        } else {
            ph_index += 1;
            ph_index - 1
        };

        ph_indices.push(index);

        if visited[index] {
            let name = fields[index].ident.to_string();
            let msg = format!("field `{}` specified more than once", name);
            error.with_error(ph.src.error(&msg));
            continue;
        }
        visited[index] = true;
    }

    if !error.is_empty() {
        return error.build_err();
    }

    let mut regex_parts = RegexParts::new(&format, &ph_indices, &fields, &types, false)?;

    error = Error::builder();
    for ((visited, field), default) in visited.iter().zip(fields.into_iter()).zip(defaults.iter()) {
        if *visited {
            continue;
        }
        if let Some(default) = default.as_ref() {
            let ident = &field.ident;
            regex_parts
                .from_matches_builder
                .push(quote!(#ident: #default));
        } else {
            let msg = format!("field `{}` has no format or default value", field.ident);
            error.with_spanned(field.ident, msg);
        }
    }
    if !error.is_empty() {
        return error.build_err();
    }

    let regex = regex_parts.regex();
    let regex_impl = quote!(
        impl ::sscanf::RegexRepresentation for #name {
            const REGEX: &'static str = #regex;
        }
    );

    let num_captures = regex_parts.num_captures();
    let from_matches = regex_parts.from_matches();
    let from_scanf_impl = quote!(
        impl ::sscanf::FromScanf for #name {
            type Err = ::sscanf::FromScanfFailedError;
            const NUM_CAPTURES: usize = #num_captures;
            fn from_matches(src: &mut ::sscanf::regex::SubCaptureMatches) -> ::std::result::Result<Self, Self::Err> {
                let start_len = src.len();
                src.next().unwrap(); // skip the full match
                let mut len = src.len();

                let mut catcher = || -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
                    Ok(#name #from_matches)
                };
                let res = catcher().map_err(|error| ::sscanf::FromScanfFailedError {
                    type_name: stringify!(#name),
                    error,
                })?;
                let n = start_len - src.len();
                if n != Self::NUM_CAPTURES {
                    panic!(
                        "{}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#name), Self::NUM_CAPTURES, n, #WRONG_CAPTURES_HINT
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

pub fn parse_enum(name: Ident, _attrs: Vec<Attribute>, _data: DataEnum) -> Result<TokenStream> {
    return Err(Error::new_spanned(name, "todo"));
}

pub fn parse_union(name: Ident, _attrs: Vec<Attribute>, _data: DataUnion) -> Result<TokenStream> {
    return Err(Error::new_spanned(name, "union is yet not supported"));
}
