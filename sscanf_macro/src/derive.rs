use std::{collections::HashMap, convert::TryFrom};

use proc_macro2::TokenStream;
use quote::quote;

use crate::*;

pub fn find_attr(attrs: Vec<syn::Attribute>) -> Result<AttributeArgMap> {
    let mut ret = AttributeArgMap::new();
    let iter = attrs.into_iter().filter(|a| a.path.is_ident("sscanf"));
    for attr in iter {
        let map = AttributeArg::from_attrs(&attr)?;
        for (k, v) in map {
            use std::collections::hash_map::Entry;
            match ret.entry(k) {
                Entry::Occupied(e) => {
                    let msg = format!("duplicate attribute `{}`", e.key());
                    return Error::builder()
                        .with_spanned(v.src, &msg)
                        .with_spanned(&e.get().src, msg)
                        .build_err();
                }
                Entry::Vacant(e) => {
                    e.insert(v);
                }
            }
        }
    }
    Ok(ret)
}

pub struct Field {
    ident: FieldIdent,
    ty: syn::Type,
    default: Option<DefaultAttribute>,
    mapper: Option<MapperAttribute>,
    ph_index: Option<usize>,
}

pub enum FieldIdent {
    Named(syn::Ident),
    Index(proc_macro2::Literal),
}
impl FieldIdent {
    pub fn from_index(i: usize, span: Span) -> FieldIdent {
        let mut literal = proc_macro2::Literal::usize_unsuffixed(i);
        literal.set_span(span);
        FieldIdent::Index(literal)
    }
}
impl ToTokens for FieldIdent {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldIdent::Named(ident) => tokens.extend(quote!(#ident)),
            FieldIdent::Index(index) => tokens.extend(quote!(#index)),
        }
    }
}
impl std::fmt::Display for FieldIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldIdent::Named(ident) => write!(f, "{}", ident),
            FieldIdent::Index(index) => write!(f, "{}", index),
        }
    }
}

fn parse_format(
    mut configs: HashMap<String, AttributeArg>,
    raw_fields: syn::Fields,
) -> Result<(RegexParts, Vec<TokenStream>, Vec<syn::Lifetime>)> {
    let found = [
        ("format", true),
        ("format_unescaped", false),
        ("format_x", false),
    ]
    .iter()
    .filter_map(|(name, escape)| configs.remove(*name).map(|a| (a, *escape)))
    .collect::<Vec<_>>();

    let (format_src, escape) = match found.as_slice() {
        [] => {
            return Error::err(Span::call_site(), "FromScanf: missing `format` attribute");
        }
        [x] => x,
        found => {
            let mut error = Error::builder();
            for (a, _) in found {
                error.with_spanned(&a.name, "only one format attribute allowed");
            }
            return error.build_err();
        }
    };

    if !configs.is_empty() {
        let mut error = Error::builder();
        for (name, attr) in configs {
            error.with_spanned(&attr.src, format!("unknown attribute arg: {}", name));
        }
        return error.build_err();
    }

    let format_src = syn::parse2::<StrLit>(format_src.value.to_token_stream())?;

    let expect_lowercase_ident = true;
    let format = FormatString::new(format_src.to_slice(), *escape, expect_lowercase_ident)?;

    let mut fields = vec![];
    let mut field_map = HashMap::new();
    let mut str_lifetimes = vec![];
    for (i, field) in raw_fields.into_iter().enumerate() {
        let mut ty = field.ty;
        let ident = field
            .ident
            .map(FieldIdent::Named)
            .unwrap_or_else(|| FieldIdent::from_index(i, ty.span()));

        field_map.insert(ident.to_string(), i);

        let mut attr = find_attr(field.attrs)?;

        let default = attr.remove("default").map(DefaultAttribute::from);
        let has_default = default.is_some();

        let mut mapper = None;
        if let Some(map) = attr.remove("map") {
            let map = MapperAttribute::try_from(map)?;
            ty = map.ty.clone();
            mapper = Some(map);
        }

        if let (Some(def), Some(map)) = (default.as_ref(), mapper.as_ref()) {
            let msg = "FromScanf: cannot use both `default` and `map` on the same field";
            return Error::builder()
                .with_spanned(&def.src, msg)
                .with_spanned(&map.src, msg)
                .build_err();
        }

        if !attr.is_empty() {
            let mut error = Error::builder();
            for (name, attr) in attr {
                error.with_spanned(&attr.src, format!("unknown attribute arg: {}", name));
            }
            return error.build_err();
        }

        let ty = match ty {
            syn::Type::Reference(r) if r.elem.to_token_stream().to_string() == "str" => {
                if !has_default {
                    str_lifetimes.extend(r.lifetime);
                }
                *r.elem
            }
            ty => ty,
        };

        fields.push(Field {
            ident,
            ty,
            default,
            mapper,
            ph_index: None,
        });
    }

    let mut ph_types = vec![];
    let mut ph_counter = 0;
    let mut error = Error::builder();
    for (ph_index, ph) in format.placeholders.iter().enumerate() {
        let index = if let Some(name) = ph.ident.as_ref() {
            if let Ok(n) = name.text().parse::<usize>() {
                if n >= fields.len() {
                    let msg = format!("field index {} out of range of {} fields", n, fields.len());
                    error.push(name.error(&msg));
                    continue;
                }
                n
            } else if let Some(i) = field_map.get(name.text()) {
                *i
            } else {
                error.push(name.error(&format!("field `{}` not found", name.text())));
                continue;
            }
        } else {
            let n = ph_counter;
            ph_counter += 1;
            n
        };

        let field = &mut fields[index];

        if field.ph_index.is_some() {
            let msg = format!("field `{}` specified more than once", field.ident);
            error.push(ph.src.error(&msg));
            continue;
        }
        field.ph_index = Some(ph_index);
        ph_types.push(TypeSource {
            ty: field.ty.clone(),
            source: Some(ph.src.clone()),
        });
    }

    error.ok_or_build()?;

    let regex_parts = RegexParts::new(&format, &ph_types)?;

    let mut unordered_matches = vec![];
    let mut ordered_matches = vec![];

    error = Error::builder();
    for field in fields {
        let ident = field.ident;
        match (field.ph_index, field.default) {
            (None, None) => {
                let msg = format!(
                    "FromScanf: field `{}` is not specified in the format string and has no default value. You must specify exactly one of these.
The syntax for default values is: `#[sscanf(default)]` to use Default::default() or `#[sscanf(default = ...)]` to provide a custom value.",
                    ident
                );
                error.with_spanned(ident, msg);
            }
            (None, Some(default)) => {
                unordered_matches.push(quote! { #ident: #default });
            }
            (Some(index), None) => {
                let matcher = &regex_parts.matchers[index];
                let mut value = quote! { #matcher };
                if let Some(mapper) = field.mapper {
                    value = quote! { { #mapper }( #value ) };
                }
                ordered_matches.push((index, quote! { #ident: #value }));
            }
            (Some(_), Some(_)) => {
                let msg = format!(
                    "FromScanf: field `{}` has a default value but is also specified in the format string.
Only one can be specified at a time",
                    ident
                );
                error.with_spanned(ident, msg);
            }
        }
    }
    error.ok_or_build()?;

    ordered_matches.sort_by_key(|(i, _)| *i);
    let from_matches = ordered_matches
        .into_iter()
        .map(|(_, m)| m)
        .chain(unordered_matches)
        .collect();

    Ok((regex_parts, from_matches, str_lifetimes))
}

pub fn parse_struct(
    name: syn::Ident,
    generics: syn::Generics,
    attr: AttributeArgMap,
    data: syn::DataStruct,
) -> Result<TokenStream> {
    let (regex_parts, from_matches, str_lifetimes) = parse_format(attr, data.fields)?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex = regex_parts.regex();
    let regex_impl = quote!(
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    );

    let mut lifetime = syn::Lifetime::new("'__from_scanf_lifetime", Span::call_site());
    let mut is_static = false;
    if let Some(lt) = str_lifetimes.iter().find(|lt| lt.ident == "static") {
        lifetime = lt.clone();
        is_static = true;
    }

    let mut lifetimed_generics = generics.clone();
    let mut where_clause = None;
    if !is_static {
        lifetimed_generics.params.push(syn::parse_quote!(#lifetime));

        let mut inner_where = generics
            .where_clause
            .clone()
            .unwrap_or_else(|| syn::WhereClause {
                where_token: Default::default(),
                predicates: Default::default(),
            });
        for lt in str_lifetimes {
            inner_where
                .predicates
                .push(syn::parse_quote!(#lifetime: #lt));
        }
        where_clause = Some(inner_where);
    }

    let (impl_generics, _, _) = lifetimed_generics.split_for_impl();

    let num_captures = regex_parts.num_captures();
    let from_sscanf_impl = quote!(
        impl #impl_generics ::sscanf::FromScanf<#lifetime> for #name #ty_generics #where_clause {
            type Err = ::sscanf::FromScanfFailedError;
            const NUM_CAPTURES: usize = #num_captures;
            fn from_matches(src: &mut ::sscanf::regex::SubCaptureMatches<'_, #lifetime>) -> ::std::result::Result<Self, Self::Err> {
                let start_len = src.len();
                src.next().unwrap(); // skip the whole match
                let mut len = src.len();

                let mut catcher = || -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
                    ::std::result::Result::Ok(#name {
                        #(#from_matches),*
                    })
                };
                let res = catcher().map_err(|error| ::sscanf::FromScanfFailedError {
                    type_name: stringify!(#name),
                    error,
                })?;
                let n = start_len - src.len();
                if n != Self::NUM_CAPTURES {
                    panic!(
                        "{}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#name), Self::NUM_CAPTURES, n, ::sscanf::WRONG_CAPTURES_HINT
                    );
                }
                Ok(res)
            }
        }
    );

    Ok(quote!(
        #regex_impl
        #from_sscanf_impl
    ))
}

#[allow(unused)] // TODO: remove
pub fn parse_enum(
    name: syn::Ident,
    generics: syn::Generics,
    attr: AttributeArgMap,
    data: syn::DataEnum,
) -> Result<TokenStream> {
    let msg = "FromScanf: enum support will be added in the next version";
    Error::err_spanned(name, msg)
}

pub fn parse_union(
    name: syn::Ident,
    _generics: syn::Generics,
    _attr: AttributeArgMap,
    _data: syn::DataUnion,
) -> Result<TokenStream> {
    let msg = "FromScanf: unions not supported yet";
    Error::err_spanned(name, msg)
}
