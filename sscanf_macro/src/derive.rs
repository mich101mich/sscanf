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
            FieldIdent::Named(ident) => write!(f, "{ident}"),
            FieldIdent::Index(index) => write!(f, "{index}"),
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

    fn get(&self, field_ty: &Type, converters: &[Parser]) -> TokenStream {
        match self {
            ValueSource::Default { def, .. } => def
                .as_ref()
                .map(|expr| quote! { #expr })
                .unwrap_or_else(|| {
                    field_ty
                        .full_span()
                        .apply(quote! { ::std::default::Default }, quote! { ::default() })
                }),
            ValueSource::Placeholder(i) => converters[*i].to_token_stream(),
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
    fn apply(&self, value: TokenStream) -> TokenStream {
        let ValueConversion { from, to, with } = self;
        use ValueConversionMethod::*;
        match with {
            From => FullSpan::from_spanned(from).apply(
                quote! { <#to as ::std::convert::From<#from>> },
                quote! { ::from(#value) },
            ),
            TryFrom => FullSpan::from_spanned(from).apply(
                quote! { <#to as ::std::convert::TryFrom<#from>> },
                quote! { ::try_from(#value).ok()? },
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
                    f(#value)?
                }}
            }
        }
    }
}

fn parse_format(
    attr: StructAttribute,
    raw_fields: syn::Fields,
) -> Result<(SequenceMatcher, TokenStream, HashSet<syn::Lifetime>)> {
    let (value, escape) = match attr.kind {
        StructAttributeKind::Format { value, escape } => (value, escape),
        StructAttributeKind::Transparent => {
            assert_or_bail!(raw_fields.len() == 1, attr.src => "structs or variants marked as `{}` must have exactly one field", attr::Struct::Transparent);
            // checked in tests/fail/derive_struct_attributes.rs

            let lit = syn::LitStr::new("{}", attr.src.span());
            (StrLit::new(lit), true)
        }
    };
    let format = FormatString::new(value.to_slice())?;

    let mut fields = vec![];
    let mut field_map = HashMap::new();
    let str_lifetimes = HashSet::new();
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

        let ty = Type::from_field(ty, ident.to_string());

        fields.push(Field {
            ident,
            ty,
            value_source,
            conversion,
        });
    }

    let mut ph_to_field_map = vec![0; format.placeholders.len()];
    let mut error = ErrorBuilder::new();
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
    let matcher = SequenceMatcher::new(&format, &ph_types, escape)?;

    let mut from_matches = vec![];

    // types from placeholders have to be extracted in order, since they rely on the iterator
    // => sort them by placeholder index, with defaults at the end
    let n = fields.len();
    fields.sort_by_key(|f| f.value_source.as_ref().map(|s| s.index_or(n)));

    for field in fields {
        let ident = field.ident;
        let ty = field.ty;

        let mut value = field.value_source.unwrap().get(&ty, &matcher.parsers);
        // unwrap is safe because the unused_field_iter above ensures that all fields have a value_source

        if let Some(conv) = field.conversion {
            value = conv.apply(value);
        }

        from_matches.push(quote! { #ident: #value });
    }
    error.ok_or_build()?;

    let parser = quote! { { #(#from_matches),* } };

    Ok((matcher, parser, str_lifetimes))
}

// TODO: depend on all the lifetimes
fn merge_lifetimes(
    str_lifetimes: HashSet<syn::Lifetime>,
    src_generics: &syn::Generics,
) -> (syn::Lifetime, syn::Generics) {
    let mut lifetime = syn::Lifetime::new("'input", Span::call_site());
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
            if lt.ident != "static" && lt.ident != lifetime.ident {
                where_clause.push(syn::parse_quote! { #lifetime: #lt });
            }
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
    let Some(attr) = StructAttribute::from_attrs(attrs)? else {
        let mut hint = "";
        if data.fields.len() == 1 {
            hint = ".
Alternatively, you can use #[sscanf(transparent)] to derive FromScanf for a single-field struct"
        }
        bail!(name => r#"FromScanf: structs must have a format string as an attribute.
Please add either of #[sscanf(format = "...")], #[sscanf(format_unescaped = "...")] or #[sscanf("...")]{hint}"#); // checked in tests/fail/derive_struct_attributes.rs
    };

    let (regex_parts, from_matches, str_lifetimes) = parse_format(attr, data.fields)?;

    let ty_generics = generics.split_for_impl().1; // generics of the type have to be kept as-is from the struct definition

    let (lifetime, lt_generics) = merge_lifetimes(str_lifetimes, generics);
    let (impl_generics, _, where_clause) = lt_generics.split_for_impl();

    let matcher = regex_parts.get_matcher();
    let from_sscanf_impl = quote! {
        #[automatically_derived]
        impl #impl_generics ::sscanf::FromScanf<#lifetime> for #name #ty_generics #where_clause {
            fn get_matcher(_: &::sscanf::advanced::FormatOptions) -> ::sscanf::advanced::Matcher {
                #matcher
            }

            fn from_match_tree(src: ::sscanf::advanced::MatchTree<'_, #lifetime>, _: &::sscanf::advanced::FormatOptions) -> ::std::option::Option<Self> {
                // TODO: add assertion for the number of matches
                struct __SscanfTokenExtensionWrapper<T>(T);
                ::std::option::Option::Some(Self #from_matches)
            }
        }
    };

    Ok(from_sscanf_impl)
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
        #[allow(
            unreachable_patterns,
            reason = "Currently there is only one variant, but this allows for future extensions"
        )]
        _ => None,
    });

    assert_or_bail!(!data.variants.is_empty(), name => "FromScanf: enums must have at least one variant"); // checked in tests/fail/derive_enum_attributes.rs

    let mut variant_matchers = vec![];
    let mut variant_parsers = vec![];
    let mut str_lifetimes = HashSet::new();

    let mut match_index = 0usize;
    for variant in data.variants.into_iter() {
        let variant_attr = VariantAttribute::from_attrs(variant.attrs)?;

        let variant_attr = if let Some(variant_attr) = variant_attr {
            match variant_attr.kind {
                VariantAttributeKind::Skip => continue,
                VariantAttributeKind::StructLike(kind) => {
                    StructAttribute::new(variant_attr.src, kind)
                }
            }
        } else if let Some((autogen, src)) = autogen.as_ref() {
            assert_or_bail!(variant.fields.is_empty(), variant.fields => r#"FromScanf: autogen only works if the variants have no fields.
Use `#[sscanf(format = "...")]` to specify a format for a variant with fields or `#[sscanf(skip)]` to skip a variant"#);
            // checked in tests/fail/derive_enum_attributes.rs

            autogen.create_struct_attr(&variant.ident.to_string(), src.clone())
        } else {
            continue;
        };

        let ident = variant.ident;

        let (variant_matcher, from_matches, variant_str_lifetimes) =
            parse_format(variant_attr, variant.fields)?;

        variant_matchers.push(variant_matcher.get_matcher());

        str_lifetimes.extend(variant_str_lifetimes);

        let converter = quote! {
            if let Some(src) = src.get(#match_index) {
                return ::std::option::Option::Some(Self::#ident #from_matches);
            } else // either `else if` with the next variant or `else` at the end
        };
        variant_parsers.push(converter);

        match_index += 1; // only increment if the variant is constructable from sscanf
    }

    if variant_parsers.is_empty() {
        if autogen.is_some() {
            bail!(name => "at least one variant has to be constructable from sscanf and not skipped."); // checked in tests/fail/derive_enum_attributes.rs
        } else {
            bail!(name => "at least one variant has to be constructable from sscanf.
To do this, add #[sscanf(format = \"...\")] to a variant"); // checked in tests/fail/derive_enum_attributes.rs
        }
    }

    let ty_generics = generics.split_for_impl().1; // generics of the type have to be kept as-is from the enum definition

    let (lifetime, lt_generics) = merge_lifetimes(str_lifetimes, generics);
    let (impl_generics, _, where_clause) = lt_generics.split_for_impl();

    let from_sscanf_impl = quote! {
        #[automatically_derived]
        impl #impl_generics ::sscanf::FromScanf<#lifetime> for #name #ty_generics #where_clause {
            fn get_matcher(_: &::sscanf::advanced::FormatOptions) -> ::sscanf::advanced::Matcher {
                ::sscanf::advanced::Matcher::from_alternation(vec![ #(#variant_matchers),* ])
            }

            fn from_match_tree(src: ::sscanf::advanced::MatchTree<'_, #lifetime>, _: &::sscanf::advanced::FormatOptions) -> ::std::option::Option<Self> {
                // TODO: add assertion for the number of matches
                struct __SscanfTokenExtensionWrapper<T>(T);
                #(#variant_parsers)* {
                    panic!("FromScanf: no variant matched");
                }
            }
        }
    };

    Ok(from_sscanf_impl)
}

pub fn parse_union(
    name: &syn::Ident,
    _generics: &syn::Generics,
    _attrs: Vec<syn::Attribute>,
    _data: syn::DataUnion,
) -> Result<TokenStream> {
    bail!(name => "FromScanf: unions not supported yet");
}
