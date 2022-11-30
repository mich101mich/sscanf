use std::collections::{HashMap, HashSet};

use crate::*;

pub struct Field {
    ident: FieldIdent,
    ty: syn::Type,
    attr: Option<FieldAttributes>,
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
            FieldIdent::Named(ident) => tokens.extend(quote! { #ident }),
            FieldIdent::Index(index) => tokens.extend(quote! { #index }),
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
    attr: StructAttributes,
    raw_fields: syn::Fields,
) -> Result<(RegexParts, Vec<TokenStream>, HashSet<syn::Lifetime>)> {
    let format = FormatString::new(attr.format.to_slice(), attr.escape)?;

    let mut fields = vec![];
    let mut field_map = HashMap::new();
    let mut str_lifetimes = HashSet::new();
    for (i, field) in raw_fields.into_iter().enumerate() {
        let mut ty = field.ty;
        let ident = field
            .ident
            .map(FieldIdent::Named)
            .unwrap_or_else(|| FieldIdent::from_index(i, ty.span()));

        field_map.insert(ident.to_string(), i);

        let attr = FieldAttributes::new(field.attrs)?;

        let mut has_default = false;
        if let Some(attr) = attr.as_ref() {
            match &attr.kind {
                FieldAttributeKind::Map {
                    ty: actual_type, ..
                } => {
                    ty = actual_type.clone();
                }
                FieldAttributeKind::Default(_) => {
                    has_default = true;
                }
            }
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
            attr,
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
                    error.push(name.error(&msg)); // checked in tests/fail/derive_placeholders.rs
                    continue;
                }
                n
            } else if let Some(i) = field_map.get(name.text()) {
                *i
            } else {
                let msg = format!("field `{}` does not exist", name.text());
                error.push(name.error(&msg)); // checked in tests/fail/derive_placeholders.rs
                continue;
            }
        } else {
            let n = ph_counter;
            if n >= fields.len() {
                let msg = format!("too many placeholders");
                error.push(ph.src.error(&msg)); // checked in tests/fail/derive_placeholders.rs
                continue;
            }
            ph_counter += 1;
            n
        };

        let field = &mut fields[index];

        if let Some(existing) = field.ph_index.as_ref() {
            let msg = format!("field `{}` specified more than once", field.ident);
            error.push(format.placeholders[*existing].src.error(&msg)); // checked in tests/fail/derive_placeholders.rs
            error.push(ph.src.error(&msg)); // checked in tests/fail/derive_placeholders.rs
            continue;
        }
        field.ph_index = Some(ph_index);
        ph_types.push(TypeSource {
            ty: field.ty.clone(),
            source: None,
        });
    }

    error.ok_or_build()?;

    let regex_parts = RegexParts::new(&format, &ph_types)?;

    // types from placeholders have to be extracted in order, since they rely on the iterator
    // => assign them by placeholder index; push the defaults to the end
    let mut from_matches = vec![TokenStream::new(); ph_types.len()];

    error = Error::builder();
    for field in fields {
        let ident = field.ident;
        let ty = field.ty;

        let default = field
            .attr
            .as_ref()
            .and_then(|attr| match &attr.kind {
                FieldAttributeKind::Default(expr) => Some(expr),
                _ => None,
            })
            .map(|opt| match opt {
                Some(expr) => expr.to_token_stream(),
                None => FullSpan::from_spanned(&ty)
                    .apply(quote! { ::std::default::Default }, quote! { ::default() }),
            });

        match (field.ph_index, default) {
            (None, None) => {
                let msg = format!(
                    "FromScanf: field `{}` is not specified in the format string and has no default value. You must specify exactly one of these.
The syntax for default values is: `#[sscanf(default)]` to use Default::default() or `#[sscanf(default = ...)]` to provide a custom value.",
                    ident
                );
                error.with_spanned(ident, msg); // checked in tests/fail/<channel>/derive_placeholders.rs
            }
            (None, Some(default)) => {
                from_matches.push(quote! { #ident: #default });
            }
            (Some(index), None) => {
                let matcher = &regex_parts.matchers[index];
                let mut value = quote! { #matcher };
                if let Some(attr) = field.attr {
                    match attr.kind {
                        FieldAttributeKind::Map { mapper, .. } => {
                            let span = mapper.body.span();
                            value = quote::quote_spanned! {span=> { #mapper }( #value ) };
                        }
                        _ => {}
                    }
                }
                from_matches[index] = quote! { #ident: #value };
            }
            (Some(index), Some(_)) => {
                let msg = format!(
                    "field `{}` has a default value but is also specified in the format string.
Only one can be specified at a time",
                    ident
                );
                error.push(format.placeholders[index].src.error(&msg)); // checked in tests/fail/<channel>/derive_placeholders.rs
                error.with_spanned(field.attr.unwrap().src, &msg); // checked in tests/fail/<channel>/derive_placeholders.rs
            }
        }
    }
    error.ok_or_build()?;

    Ok((regex_parts, from_matches, str_lifetimes))
}

fn merge_lifetimes(
    str_lifetimes: HashSet<syn::Lifetime>,
    src_generics: &syn::Generics,
) -> (syn::Lifetime, syn::Generics) {
    let mut lifetime = syn::Lifetime::new("'__from_scanf_lifetime", Span::call_site());
    let mut is_static = false;
    if let Some(lt) = str_lifetimes.iter().find(|lt| lt.ident == "static") {
        lifetime = lt.clone();
        is_static = true;
    }

    let mut lifetimed_generics = src_generics.clone();
    if !is_static {
        lifetimed_generics.params.push(syn::parse_quote!(#lifetime));

        let mut inner_where =
            src_generics
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
        lifetimed_generics.where_clause = Some(inner_where);
    }

    (lifetime, lifetimed_generics)
}

pub fn parse_struct(
    name: syn::Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
    data: syn::DataStruct,
) -> Result<TokenStream> {
    let attr = StructAttributes::new(attrs)?;
    let (regex_parts, from_matches, str_lifetimes) = match attr {
        Some(attr) => parse_format(attr, data.fields)?,
        None => {
            let msg = "FromScanf: structs must have a format string as an attribute.
Please add either of #[sscanf(format = \"...\")], #[sscanf(format_unescaped = \"...\")] or #[sscanf(\"...\")]";
            return Error::err_spanned(name, msg); // checked in tests/fail/derive_struct_attributes.rs
        }
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex = regex_parts.regex();
    let regex_impl = quote! {
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    };

    let (lifetime, lt_generics) = merge_lifetimes(str_lifetimes, &generics);
    let (impl_generics, _, where_clause) = lt_generics.split_for_impl();

    let num_captures = regex_parts.num_captures();
    let from_sscanf_impl = quote! {
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
                        "sscanf: {}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#name), Self::NUM_CAPTURES, n, ::sscanf::WRONG_CAPTURES_HINT
                    );
                }
                Ok(res)
            }
        }
    };

    Ok(quote! {
        #regex_impl
        #from_sscanf_impl
    })
}

pub fn parse_enum(
    name: syn::Ident,
    generics: syn::Generics,
    attrs: Vec<syn::Attribute>,
    data: syn::DataEnum,
) -> Result<TokenStream> {
    let attrs = attrs
        .iter()
        .filter(|attr| attr.path.is_ident("sscanf"))
        .collect::<Vec<_>>();
    if !attrs.is_empty() {
        let msg = "enums cannot have outer attributes";
        let mut error = Error::builder();
        for attr in attrs {
            error.with_spanned(attr, msg);
        }
        return error.build_err(); // checked in tests/fail/derive_struct_attributes.rs
    }

    let mut regex_parts = RegexParts::empty();
    regex_parts.push_literal("(?:");
    let mut variant_constructors = vec![];
    let mut num_captures_list = vec![NumCaptures::One];
    let mut str_lifetimes = HashSet::new();
    let mut first = true;

    for variant in data.variants.into_iter() {
        let attr = match StructAttributes::new(variant.attrs)? {
            Some(attr) => attr,
            None => continue,
        };

        if first {
            regex_parts.push_literal("(");
            first = false;
        } else {
            regex_parts.push_literal("|(");
        };

        let ident = variant.ident;

        let (variant_parts, from_matches, variant_str_lifetimes) =
            parse_format(attr, variant.fields)?;

        let variant_num_captures_list = variant_parts.num_captures_list();
        let num_captures = quote! { #(#variant_num_captures_list)+* };
        num_captures_list.extend(variant_num_captures_list);

        regex_parts
            .regex_builder
            .extend(variant_parts.regex_builder);

        str_lifetimes.extend(variant_str_lifetimes);

        let matcher = quote! {
            #[cfg(debug_assertions)]
            let start_len = src.len();

            let expected = #num_captures;

            if src.next().expect(::sscanf::EXPECT_NEXT_HINT).is_some() && ret.is_none() {
                ret = Some(#name::#ident {
                    #(#from_matches),*
                });
            } else if expected > 1 {
                src.nth(expected - 2).expect(::sscanf::EXPECT_NEXT_HINT);
            }

            #[cfg(debug_assertions)]
            {
                let n = start_len - src.len();
                if n != expected {
                    panic!(
                        "sscanf: {}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#ident), expected, n, ::sscanf::WRONG_CAPTURES_HINT
                    );
                }
            }
        };
        variant_constructors.push(matcher);

        regex_parts.push_literal(")");
    }
    regex_parts.push_literal(")");

    if variant_constructors.is_empty() {
        let msg = "at least one variant has to be constructable from sscanf.
To do this, add #[sscanf(format = \"...\")] to a variant";
        return Error::err_spanned(name, msg); // checked in tests/fail/derive_struct_attributes.rs
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex = regex_parts.regex();
    let regex_impl = quote! {
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    };

    let (lifetime, lt_generics) = merge_lifetimes(str_lifetimes, &generics);
    let (impl_generics, _, where_clause) = lt_generics.split_for_impl();

    let from_sscanf_impl = quote! {
        impl #impl_generics ::sscanf::FromScanf<#lifetime> for #name #ty_generics #where_clause {
            type Err = ::sscanf::FromScanfFailedError;

            const NUM_CAPTURES: usize = #(#num_captures_list)+*;

            fn from_matches(src: &mut ::sscanf::regex::SubCaptureMatches<'_, #lifetime>) -> ::std::result::Result<Self, Self::Err> {
                let start_len = src.len();
                src.next().unwrap(); // skip the whole match
                let mut len = src.len();

                let mut catcher = || -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
                    let mut ret: ::std::option::Option<Self> = ::std::option::Option::None;

                    #(#variant_constructors)*

                    let ret = ret.expect("sscanf: enum regex matched but no variant was captured");
                    ::std::result::Result::Ok(ret)
                };
                let res = catcher().map_err(|error| ::sscanf::FromScanfFailedError {
                    type_name: stringify!(#name),
                    error,
                })?;
                let n = start_len - src.len();
                if n != Self::NUM_CAPTURES {
                    panic!(
                        "sscanf: {}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#name), Self::NUM_CAPTURES, n, ::sscanf::WRONG_CAPTURES_HINT
                    );
                }
                Ok(res)
            }
        }
    };

    Ok(quote! {
        #regex_impl
        #from_sscanf_impl
    })
}

pub fn parse_union(
    name: syn::Ident,
    _generics: syn::Generics,
    _attrs: Vec<syn::Attribute>,
    _data: syn::DataUnion,
) -> Result<TokenStream> {
    let msg = "FromScanf: unions not supported yet";
    Error::err_spanned(name, msg)
}
