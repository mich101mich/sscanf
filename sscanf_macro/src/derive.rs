use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Attribute, DataEnum, DataStruct, DataUnion, Generics, Ident};

use crate::*;

pub fn find_attr(attrs: Vec<Attribute>) -> Option<Attribute> {
    attrs.into_iter().find(|a| a.path.is_ident("scanf"))
}

fn parse_format(
    configs: &mut HashMap<String, FormatAttribute>,
    configs_src: Attribute,
    raw_fields: syn::Fields,
) -> Result<RegexParts> {
    let found = [
        ("format", true),
        ("format_unescaped", false),
        ("format_x", false),
    ]
    .iter()
    .filter_map(|(name, escape)| configs.remove(*name).map(|a| (a, *escape)))
    .collect::<Vec<_>>();

    let (format_src, escape) = match found.len() {
        0 => {
            return Error::err_spanned(configs_src, "FromScanf: missing `format` attribute");
        }
        1 => found.into_iter().next().unwrap(),
        _ => {
            let mut error = Error::builder();
            for (a, _) in found {
                error.with_spanned(a.name, "FromScanf: only one format attribute allowed");
            }
            return error.build_err();
        }
    };

    if !configs.is_empty() {
        let mut error = Error::builder();
        for (name, attr) in configs {
            error.with_spanned(&attr.src, format!("FromScanf: unknown attribute: {}", name));
        }
        return error.build_err();
    }

    let format = FormatString::new(format_src.value.to_slice(), escape)?;

    let mut fields = vec![];
    let mut field_map = HashMap::new();
    let mut defaults = vec![];
    let mut types = vec![];
    for (i, field) in raw_fields.into_iter().enumerate() {
        let ty = field.ty;
        let ident = field
            .ident
            .map(FieldIdent::Named)
            .unwrap_or_else(|| FieldIdent::from_index(i, ty.span()));

        field_map.insert(ident.to_string(), i);

        let mut default = None;

        if let Some(attr) = find_attr(field.attrs) {
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

    if format.placeholders.len() != fields.len() {
        let msg = format!(
            "FromScanf: format string has {} placeholders, but struct has {} fields",
            format.placeholders.len(),
            fields.len()
        );
        return Error::err_spanned(format_src.src, msg);
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

    error.ok_or_build()?;

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
            let msg = format!(
                "FromScanf: field `{}` has no format or default value",
                field.ident
            );
            error.with_spanned(field.ident, msg);
        }
    }
    error.ok_or_build()?;

    Ok(regex_parts)
}

pub fn parse_struct(
    name: Ident,
    generics: Generics,
    attr: Option<Attribute>,
    data: DataStruct,
) -> Result<TokenStream> {
    let attr = attr.ok_or_else(|| {
        let msg = "FromScanf: structs must have a #[scanf(format=\"...\")] attribute";
        Error::new_spanned(&name, msg)
    })?;

    let mut configs = FormatAttribute::from_attrs(&attr)?;

    let regex_parts = parse_format(&mut configs, attr, data.fields)?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex = regex_parts.regex();
    let regex_impl = quote!(
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    );

    let num_captures = regex_parts.num_captures();
    let from_matches = regex_parts.from_matches();
    let from_scanf_impl = quote!(
        impl #impl_generics ::sscanf::FromScanf for #name #ty_generics #where_clause {
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

pub fn parse_enum(
    name: Ident,
    generics: Generics,
    attr: Option<Attribute>,
    data: DataEnum,
) -> Result<TokenStream> {
    if let Some(attr) = attr {
        let msg = "FromScanf: enum formats have to be specified per-variant";
        return Error::err_spanned(attr, msg);
    }

    let mut variants = vec![];
    let mut error = Error::builder();
    for variant in data.variants.into_iter() {
        if let Some(attr) = find_attr(variant.attrs) {
            variants.push((variant.ident, variant.fields, attr));
        }
    }
    error.ok_or_build()?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex_parts = RegexParts::empty();

    // TODO: fill regex_parts

    let regex = regex_parts.regex();
    let regex_impl = quote!(
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    );

    let num_captures = regex_parts.num_captures();
    let from_matches = regex_parts.from_matches();
    let from_scanf_impl = quote!(
        impl #impl_generics ::sscanf::FromScanf for #name #ty_generics #where_clause {
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

pub fn parse_union(
    name: Ident,
    _generics: Generics,
    _attr: Option<Attribute>,
    _data: DataUnion,
) -> Result<TokenStream> {
    let msg = "FromScanf: unions not supported yet";
    return Err(Error::new_spanned(name, msg));
}
