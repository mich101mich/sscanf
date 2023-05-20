use std::collections::{HashMap, HashSet};

use crate::*;

pub struct Field {
    ident: FieldIdent,
    ty: Type<'static>,
    value_source: Option<ValueSource>,
    conversion: Option<ValueConversion>,
}

enum FieldIdent {
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

enum ValueSource {
    Default {
        def: Option<syn::Expr>,
        src: TokenStream,
    },
    Placeholder(usize),
}
impl ValueSource {
    fn error<U: std::fmt::Display>(&self, msg: U, placeholders: &[Placeholder]) -> Error {
        match self {
            ValueSource::Default { src, .. } => Error::new_spanned(src, msg),
            ValueSource::Placeholder(i) => placeholders[*i].src.error(msg),
        }
    }

    fn get(&self, field_ty: &Type, matchers: &[Matcher]) -> TokenStream {
        match self {
            ValueSource::Default { def, .. } => def
                .as_ref()
                .map(|expr| quote! { #expr })
                .unwrap_or_else(|| {
                    field_ty
                        .full_span()
                        .apply(quote! { ::std::default::Default }, quote! { ::default() })
                }),
            ValueSource::Placeholder(i) => {
                let matcher = &matchers[*i];
                quote! { #matcher }
            }
        }
    }

    fn index_or(&self, n: usize) -> usize {
        match self {
            ValueSource::Default { .. } => n,
            ValueSource::Placeholder(i) => *i,
        }
    }
}

struct ValueConversion {
    from: syn::Type,
    to: syn::Type,
    with: ValueConversionMethod,
}
enum ValueConversionMethod {
    From,
    TryFrom,
    Map(syn::ExprClosure),
    FilterMap(syn::ExprClosure),
}
impl ValueConversion {
    fn apply(&self, value: TokenStream, field_name: &FieldIdent) -> TokenStream {
        let ValueConversion { from, to, with } = self;
        use ValueConversionMethod::*;
        match with {
            From => FullSpan::from_spanned(from).apply(
                quote! { <#to as ::std::convert::From<#from>> },
                quote! { ::from(#value) },
            ),
            TryFrom => FullSpan::from_spanned(from).apply(
                quote! { <#to as ::std::convert::TryFrom<#from>> },
                quote! { ::try_from(#value)? },
            ),
            Map(closure) => {
                let span = closure.body.span();
                quote::quote_spanned! {span=> {
                    let f: fn(#from) -> #to = #closure;
                    f(#value)
                }}
            }
            FilterMap(closure) => {
                let span = closure.body.span();
                quote::quote_spanned! {span=> {
                    let f: fn(#from) -> ::std::option::Option<#to> = #closure;
                    f(#value).ok_or(::sscanf::errors::FilterMapNoneError {
                        field_name: stringify!(#field_name)
                    })?
                }}
            }
        }
    }
}

fn parse_format(
    attr: StructAttribute,
    raw_fields: syn::Fields,
) -> Result<(RegexParts, TokenStream, HashSet<syn::Lifetime>)> {
    let (value, escape) = match attr.kind {
        StructAttributeKind::Format { value, escape } => (value, escape),
        StructAttributeKind::Transparent => {
            if raw_fields.len() != 1 {
                let msg = format!(
                    "structs or variants marked as `{}` must have exactly one field",
                    attr::Struct::Transparent
                );
                return Error::err_spanned(attr.src, msg); // checked in tests/fail/derive_struct_attributes.rs
            }
            let lit = syn::LitStr::new("{}", attr.src.span());
            (StrLit::new(lit), true)
        }
    };
    let format = FormatString::new(value.to_slice(), escape)?;

    let mut fields = vec![];
    let mut field_map = HashMap::new();
    let mut str_lifetimes = HashSet::new();
    for (i, field) in raw_fields.into_iter().enumerate() {
        let mut ty = field.ty;
        let ident = if let Some(ident) = field.ident {
            field_map.insert(ident.to_string(), i);
            FieldIdent::Named(ident)
        } else {
            FieldIdent::from_index(i, ty.span())
        };
        field_map.insert(i.to_string(), i);

        let attr = FieldAttribute::from_attrs_with(field.attrs, &ty)?;

        let mut value_source = None;
        let mut conversion = None;
        if let Some(attr) = attr {
            use FieldAttributeKind as AttrKind;
            use ValueConversionMethod as Conv;
            match attr.kind {
                AttrKind::Default(def) => {
                    value_source = Some(ValueSource::Default { def, src: attr.src });
                }
                AttrKind::Map {
                    mapper,
                    ty: from,
                    filters,
                } => {
                    let to = std::mem::replace(&mut ty, from.clone());
                    let conv = if filters { Conv::FilterMap } else { Conv::Map };
                    let with = conv(mapper);
                    conversion = Some(ValueConversion { from, to, with });
                }
                AttrKind::From { ty: from, tries } => {
                    let to = std::mem::replace(&mut ty, from.clone());
                    let with = if tries { Conv::TryFrom } else { Conv::From };
                    conversion = Some(ValueConversion { from, to, with });
                }
            }
        }

        let ty = Type::from_ty(ty);

        if value_source.is_none() {
            if let Some(lt) = ty.lifetime() {
                str_lifetimes.insert(lt.clone());
            }
        }

        fields.push(Field {
            ident,
            ty,
            value_source,
            conversion,
        });
    }

    let mut ph_to_field_map = vec![0; format.placeholders.len()];
    let mut error = Error::builder();
    for (ph_index, ph) in format.placeholders.iter().enumerate() {
        let name = match ph.ident.as_ref() {
            Some(name) => name,
            None => continue,
        };

        let index = if let Some(i) = field_map.get(name.text()) {
            *i
        } else {
            let msg = if let Ok(n) = name.text().parse::<usize>() {
                // checked in tests/fail/derive_placeholders.rs
                format!("field index {} out of range of {} fields", n, fields.len())
            } else {
                // checked in tests/fail/derive_placeholders.rs
                format!("field `{}` does not exist", name.text())
            };
            error.push(name.error(msg));
            continue;
        };

        let field = &mut fields[index];

        if let Some(existing) = field.value_source.as_ref() {
            let msg = format!("field `{}` has multiple sources", name.text());
            error.push(existing.error(&msg, &format.placeholders)); // checked in tests/fail/derive_placeholders.rs
            error.push(ph.src.error(&msg)); // checked in tests/fail/derive_placeholders.rs
            continue;
        }
        field.value_source = Some(ValueSource::Placeholder(ph_index));
        ph_to_field_map[ph_index] = index;
    }

    let mut unused_field_iter = fields
        .iter_mut()
        .enumerate()
        .filter(|(_, f)| f.value_source.is_none());

    for (ph_index, ph) in format.placeholders.iter().enumerate() {
        if ph.ident.is_some() {
            continue;
        }
        let (index, field) = if let Some(val) = unused_field_iter.next() {
            val
        } else {
            let msg = "too many placeholders";
            error.push(ph.src.error(msg)); // checked in tests/fail/derive_placeholders.rs
            continue;
        };
        // field.ph_index is guaranteed to be None because of the iterator filter
        field.value_source = Some(ValueSource::Placeholder(ph_index));
        ph_to_field_map[ph_index] = index;
    }

    for (_, unused) in unused_field_iter {
        let msg = format!(
            "FromScanf: field `{}` is not specified in the format string and has no default value. You must specify exactly one of these.
The syntax for default values is: `#[sscanf(default)]` to use Default::default() or `#[sscanf(default = ...)]` to provide a custom value.",
            unused.ident
        );
        error.with_spanned(&unused.ident, msg); // checked in tests/fail/<channel>/derive_placeholders.rs
    }

    error.ok_or_build()?;

    let ph_types = ph_to_field_map
        .iter()
        .map(|i| fields[*i].ty.clone())
        .collect::<Vec<_>>();
    let regex_parts = RegexParts::new(&format, &ph_types)?;

    let mut from_matches = vec![];

    // types from placeholders have to be extracted in order, since they rely on the iterator
    // => sort them by placeholder index, with defaults at the end
    let n = fields.len();
    fields.sort_by_key(|f| f.value_source.as_ref().map(|s| s.index_or(n)));

    for field in fields {
        let ident = field.ident;
        let ty = field.ty;

        let mut value = field.value_source.unwrap().get(&ty, &regex_parts.matchers);
        // unwrap is safe because the unused_field_iter above ensures that all fields have a value_source

        if let Some(conv) = field.conversion {
            value = conv.apply(value, &ident);
        }

        from_matches.push(quote! { #ident: #value });
    }
    error.ok_or_build()?;

    let from_matches = quote! { { #(#from_matches),* } };

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
        lifetimed_generics
            .params
            .push(syn::parse_quote! { #lifetime });

        let where_clause = &mut lifetimed_generics.make_where_clause().predicates;
        for lt in str_lifetimes {
            where_clause.push(syn::parse_quote! { #lifetime: #lt });
        }
    }

    (lifetime, lifetimed_generics)
}

pub fn parse_struct(
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: Vec<syn::Attribute>,
    data: syn::DataStruct,
) -> Result<TokenStream> {
    let attr = StructAttribute::from_attrs(attrs)?
        .ok_or_else(|| {
            let mut msg = "FromScanf: structs must have a format string as an attribute.
Please add either of #[sscanf(format = \"...\")], #[sscanf(format_unescaped = \"...\")] or #[sscanf(\"...\")]".to_string();
            if data.fields.len() == 1 {
                msg += ".
Alternatively, you can use #[sscanf(transparent)] to derive FromScanf for a single-field struct";
            }
            Error::new_spanned(name, msg) // checked in tests/fail/derive_struct_attributes.rs
        })?;

    let (regex_parts, from_matches, str_lifetimes) = parse_format(attr, data.fields)?;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex = regex_parts.regex();
    let regex_impl = quote! {
        #[automatically_derived]
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    };

    let (lifetime, lt_generics) = merge_lifetimes(str_lifetimes, generics);
    let (impl_generics, _, where_clause) = lt_generics.split_for_impl();

    let num_captures = regex_parts.num_captures();
    let from_sscanf_impl = quote! {
        #[automatically_derived]
        impl #impl_generics ::sscanf::FromScanf<#lifetime> for #name #ty_generics #where_clause {
            type Err = ::sscanf::errors::FromScanfFailedError;
            const NUM_CAPTURES: usize = #num_captures;
            fn from_matches(src: &mut ::sscanf::regex::SubCaptureMatches<'_, #lifetime>) -> ::std::result::Result<Self, Self::Err> {
                let start_len = src.len();
                src.next().unwrap(); // skip the whole match
                let mut len = src.len();

                let mut catcher = || -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
                    ::std::result::Result::Ok(#name #from_matches)
                };
                let res = catcher().map_err(|error| ::sscanf::errors::FromScanfFailedError {
                    type_name: stringify!(#name),
                    error,
                })?;
                let n = start_len - src.len();
                if n != Self::NUM_CAPTURES {
                    panic!(
                        "sscanf: {}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#name), Self::NUM_CAPTURES, n, ::sscanf::errors::WRONG_CAPTURES_HINT
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
    name: &syn::Ident,
    generics: &syn::Generics,
    attrs: Vec<syn::Attribute>,
    data: syn::DataEnum,
) -> Result<TokenStream> {
    let attr = EnumAttribute::from_attrs(attrs)?;
    let autogen = attr.as_ref().and_then(|attr| match attr.kind {
        EnumAttributeKind::AutoGen(kind) => Some((kind, attr.src.clone())),
        #[allow(unreachable_patterns)]
        _ => None,
    });

    if data.variants.is_empty() {
        let msg = "FromScanf: enums must have at least one variant";
        return Error::err_spanned(name, msg); // checked in tests/fail/derive_enum_attributes.rs
    }

    let mut regex_parts = RegexParts::empty();
    regex_parts.push_literal("(?:");
    let mut variant_constructors = vec![];
    let mut num_captures_list = vec![NumCaptures::One];
    let mut str_lifetimes = HashSet::new();
    let mut first = true;

    for variant in data.variants {
        let variant_attr = VariantAttribute::from_attrs(variant.attrs)?;

        let variant_attr = if let Some(variant_attr) = variant_attr {
            match variant_attr.kind {
                VariantAttributeKind::Skip => continue,
                VariantAttributeKind::StructLike(kind) => {
                    StructAttribute::new(variant_attr.src, kind)
                }
            }
        } else if let Some((autogen, src)) = autogen.as_ref() {
            if !variant.fields.is_empty() {
                let msg = "FromScanf: autogen only works if the variants have no fields.
Use `#[sscanf(format = \"...\")]` to specify a format for a variant with fields or `#[sscanf(skip)]` to skip a variant";
                return Error::err_spanned(variant.fields, msg); // checked in tests/fail/derive_enum_attributes.rs
            }
            autogen.create_struct_attr(&variant.ident.to_string(), src.clone())
        } else {
            continue;
        };

        if first {
            regex_parts.push_literal("(");
            first = false;
        } else {
            regex_parts.push_literal("|(");
        };

        let ident = variant.ident;

        let (variant_parts, from_matches, variant_str_lifetimes) =
            parse_format(variant_attr, variant.fields)?;

        let variant_num_captures_list = variant_parts.num_captures_list();
        let num_captures = quote! { #(#variant_num_captures_list)+* };
        num_captures_list.extend(variant_num_captures_list);

        regex_parts
            .regex_builder
            .extend(variant_parts.regex_builder);

        str_lifetimes.extend(variant_str_lifetimes);

        let matcher = quote! {
            let expected = #num_captures;

            remaining -= expected;
            if src.next().expect(::sscanf::errors::EXPECT_NEXT_HINT).is_some() {
                return ::std::result::Result::Ok(#name::#ident #from_matches);
            } else if expected > 1 { // one was already taken by `src.next()` above
                src.nth(expected - 2).expect(::sscanf::errors::EXPECT_NEXT_HINT);
            }
        };
        variant_constructors.push(matcher);

        regex_parts.push_literal(")");
    }
    regex_parts.push_literal(")");

    if variant_constructors.is_empty() {
        let msg = if autogen.is_some() {
            "at least one variant has to be constructable from sscanf and not skipped."
        } else {
            "at least one variant has to be constructable from sscanf.
To do this, add #[sscanf(format = \"...\")] to a variant"
        };
        return Error::err_spanned(name, msg); // checked in tests/fail/derive_enum_attributes.rs
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let regex = regex_parts.regex();
    let regex_impl = quote! {
        #[automatically_derived]
        impl #impl_generics ::sscanf::RegexRepresentation for #name #ty_generics #where_clause {
            const REGEX: &'static ::std::primitive::str = #regex;
        }
    };

    let (lifetime, lt_generics) = merge_lifetimes(str_lifetimes, generics);
    let (impl_generics, _, where_clause) = lt_generics.split_for_impl();

    let from_sscanf_impl = quote! {
        #[automatically_derived]
        impl #impl_generics ::sscanf::FromScanf<#lifetime> for #name #ty_generics #where_clause {
            type Err = ::sscanf::errors::FromScanfFailedError;

            const NUM_CAPTURES: usize = #(#num_captures_list)+*;

            fn from_matches(src: &mut ::sscanf::regex::SubCaptureMatches<'_, #lifetime>) -> ::std::result::Result<Self, Self::Err> {
                let start_len = src.len();
                let mut remaining = Self::NUM_CAPTURES;
                src.next().unwrap(); // skip the whole match
                remaining -= 1;

                let mut len = src.len();

                let mut catcher = || -> ::std::result::Result<Self, ::std::boxed::Box<dyn ::std::error::Error>> {
                    #(#variant_constructors)*

                    unreachable!("sscanf: enum regex matched but no variant was captured");
                };
                let res = catcher().map_err(|error| ::sscanf::errors::FromScanfFailedError {
                    type_name: stringify!(#name),
                    error,
                })?;

                if remaining > 0 {
                    src.nth(remaining - 1).expect(::sscanf::errors::EXPECT_NEXT_HINT);
                }

                let n = start_len - src.len();
                if n != Self::NUM_CAPTURES {
                    panic!(
                        "sscanf: {}::NUM_CAPTURES = {} but {} were taken{}",
                        stringify!(#name), Self::NUM_CAPTURES, n, ::sscanf::errors::WRONG_CAPTURES_HINT
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
    name: &syn::Ident,
    _generics: &syn::Generics,
    _attrs: Vec<syn::Attribute>,
    _data: syn::DataUnion,
) -> Result<TokenStream> {
    let msg = "FromScanf: unions not supported yet";
    Error::err_spanned(name, msg)
}
