use crate::*;

use std::collections::{HashMap, HashSet};

/// A field on a struct or struct-like enum variant
pub struct Field<'a> {
    /// The identifier of the field
    ident: FieldIdent,
    /// The type that the field should be parsed to (might be different from the output type due to conversions)
    parsed_type: Type<'static>,
    /// The source where the value for the field will come from
    value_source: ValueSource<'a>,
    /// The conversion that should be applied to the parsed value
    conversion: ValueConversion,
    /// The type that the field will be output as. The same type as the struct field type
    output_type: Type<'static>,
}
impl Field<'_> {
    pub fn to_parser(&self, parsers: &[Parser]) -> TokenStream {
        let parsed_value = self.value_source.get(&self.parsed_type, parsers);

        let converted_value =
            self.conversion
                .apply(parsed_value, &self.parsed_type, &self.output_type);

        let ident = &self.ident;
        quote! { #ident: #converted_value }
    }
}

/// An identifier for a field, either named (struct with curly brackets) or indexed (tuple struct)
///
/// This is a bit of a workaround because `syn::Ident` doesn't accept numbers as identifiers,
/// but when working with a tuple struct(-variant), we are effectively using the index as an identifier.
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
impl Display for FieldIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldIdent::Named(ident) => write!(f, "{ident}"),
            FieldIdent::Index(index) => write!(f, "{index}"),
        }
    }
}

/// The source where the value for a field will come from when parsing
enum ValueSource<'a> {
    /// Field is parsed directly from a placeholder
    Placeholder {
        /// The index of the placeholder in the format string
        index: usize,
        /// The source of the placeholder, used for error reporting
        src: StrLitSlice<'a>,
    },

    /// Field is set to a default value
    Default {
        /// The default value expression, if any. Default::default() is used if this is None
        def: Option<syn::Expr>,
        /// The source of the default value, used for error reporting
        src: TokenStream,
    },
}
impl ValueSource<'_> {
    /// Returns a token stream that returns the value for this field
    fn get(&self, field_ty: &Type, parsers: &[Parser]) -> TokenStream {
        match self {
            ValueSource::Default { def, .. } => {
                if let Some(expr) = def {
                    quote! { #expr }
                } else {
                    field_ty
                        .full_span()
                        .apply(quote! { ::std::default::Default }, quote! { ::default() })
                }
            }
            ValueSource::Placeholder { index, .. } => parsers[*index].to_token_stream(),
        }
    }

    /// Returns true if the value makes use of the parsed type
    fn uses_parsed_type(&self) -> bool {
        match self {
            ValueSource::Default { .. } => false, // default values are not parsed
            ValueSource::Placeholder { .. } => true, // placeholder values are parsed from a placeholder
        }
    }
}
impl<'a> Sourced<'a> for ValueSource<'a> {
    fn error(&self, message: impl Display) -> Error {
        match self {
            ValueSource::Default { src, .. } => src.error(message),
            ValueSource::Placeholder { src, .. } => src.error(message),
        }
    }
}

/// A conversion that should be applied to a field's value after parsing
enum ValueConversion {
    /// No conversion, just use the parsed value as-is
    None,
    /// Use the `From` trait
    From,
    /// Use the `TryFrom` trait
    TryFrom,
    /// Use a closure to map the value
    Map(syn::ExprClosure),
    /// Use a closure to filter and map the value
    FilterMap(syn::ExprClosure),
}
impl ValueConversion {
    /// Applies the conversion to the given value, converting it from `from` type to `to` type
    fn apply(&self, value: TokenStream, from: &Type, to: &Type) -> TokenStream {
        use ValueConversion::*;
        match self {
            None => value,
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
    struct_fields: syn::Fields,
) -> Result<(SequenceMatcher, TokenStream, HashSet<syn::Lifetime>)> {
    let (value, escape) = match attr.kind {
        StructAttributeKind::Format { value, escape } => (value, escape),
        StructAttributeKind::Transparent => {
            assert_or_bail!(struct_fields.len() == 1, attr => "structs or variants marked as `{}` must have exactly one field", attr::Struct::Transparent);
            // checked in tests/fail/derive_struct_attributes.rs

            let lit = syn::LitStr::new("{}", attr.src.span());
            (StrLit::new(lit), true)
        }
    };
    let format = FormatString::new(value.to_slice())?;

    // Map from the ident (field name or index) used in a placeholder to its index in the format string
    let mut explicit_ph_identifiers = HashMap::new();
    // List of indices of the placeholders that *don't* have an explicit identifier
    let mut unused_ph_indices = vec![];
    for (i, ph) in format.placeholders.iter().enumerate() {
        if let Some(ident) = &ph.ident {
            let name = ident.text();
            if let Some(prev_entry) = explicit_ph_identifiers.insert(name, i) {
                let prev = &format.placeholders[prev_entry];
                bail!({ph, prev} => "placeholder `{name}` is used multiple times in the format string"); // TODO: check
            }
        } else {
            unused_ph_indices.push(i);
        }
    }
    let mut unused_ph_indices = unused_ph_indices.into_iter(); // convert to an iterator for easier access

    // List of fields that are neither default nor provided by a placeholder
    let mut unspecified_fields = vec![];

    let mut fields = vec![];
    for (i, field) in struct_fields.into_iter().enumerate() {
        let output_type = field.ty;

        let mut ph_index = None;
        let ident = if let Some(ident) = field.ident {
            // check if the field is explicitly used in a placeholder
            if let Some(index) = explicit_ph_identifiers.remove(ident.to_string().as_str()) {
                ph_index = Some(index);
            }
            FieldIdent::Named(ident)
        } else {
            FieldIdent::from_index(i, output_type.span())
        };

        // check if the field is referred to by index in a placeholder
        if let Some(index) = explicit_ph_identifiers.remove(i.to_string().as_str()) {
            if let Some(prev_index) = ph_index {
                let prev = &format.placeholders[prev_index];
                let ph = &format.placeholders[index];
                bail!({ph, prev} => "field `{}` is used in multiple placeholders", ident); // TODO: check
            }
            ph_index = Some(index);
        }

        // if the field is not explicitly used in a placeholder, we would need to use the next unused placeholder index.
        // However, we first need to check if the field is marked as default

        let attr = FieldAttribute::from_attrs_with(field.attrs, &output_type)?;

        let mut parsed_type = output_type.clone(); // by default, these are assumed to be the same
        let mut value_source = None;
        let mut conversion = ValueConversion::None;
        if let Some(attr) = attr {
            use ValueConversion as Conv;
            match attr.kind {
                FieldAttributeKind::Default(def) => {
                    value_source = Some(ValueSource::Default { def, src: attr.src });
                }
                FieldAttributeKind::From { ty: from, tries } => {
                    parsed_type = from.clone();
                    conversion = if tries { Conv::TryFrom } else { Conv::From };
                }
                FieldAttributeKind::Map {
                    mapper,
                    ty: from,
                    filters,
                } => {
                    parsed_type = from;
                    conversion = if filters { Conv::FilterMap } else { Conv::Map }(mapper);
                }
            }
        }

        let value_source = if let Some(value_source) = value_source {
            // value is provided by an attribute
            if let Some(ph_index) = ph_index {
                let ph = &format.placeholders[ph_index];
                bail!({ph, value_source} => "field `{ident}` is used in multiple placeholders");
            }
            value_source
        } else if let Some(index) = ph_index.or_else(|| unused_ph_indices.next()) {
            // value is provided by a placeholder
            ValueSource::Placeholder {
                index,
                src: format.placeholders[index].src,
            }
        } else {
            unspecified_fields.push(ident);
            continue; // skip this field for now and print an error later
        };

        let parsed_type = Type::from_field(parsed_type, ident.to_string());
        let output_type = Type::from_ty(output_type);

        fields.push(Field {
            ident,
            parsed_type,
            value_source,
            conversion,
            output_type,
        });
    }

    let mut error = ErrorBuilder::new();

    // produce errors for any unused placeholders or unspecified fields.
    // Note that we technically can't have both and there is no real point in handling them at once, but there is the
    // possibility that the user made a typo in the placeholder identifier, so we first collect all offenders in order
    // to provide a "Did you mean..." suggestion.

    for unused in unused_ph_indices {
        // There is an empty placeholder with no matching field
        let ph = &format.placeholders[unused];
        error.push(error!(ph => "More placeholders than fields in the format string"));
    }
    for (name, index) in &explicit_ph_identifiers {
        let ph = &format.placeholders[*index];
        if let Ok(n) = name.parse::<usize>() {
            // User specified a number that seems to be out of range
            error.push(error!(ph => "No field with index {n} exists"));
        } else if let Some(closest) = take_closest(name, &mut unspecified_fields) {
            // User provided an unknown field name, but we can suggest a closest match
            error.push(error!(ph => "placeholder `{name}` does not match any field. Did you mean `{closest}`?"));
        } else {
            // User provided an unknown field name
            error.push(error!(ph => "placeholder `{name}` does not match any field"));
        }
    }
    for field in unspecified_fields {
        // There is a field that is not specified in the format string
        if explicit_ph_identifiers.is_empty() {
            error.push(error!(field => "More fields than placeholders in the format string.
Either add more placeholders or provide a default value with `#[sscanf(default)]` or `#[sscanf(default = ...)]`"));
        } else {
            error.push(error!(field => "Field {field} is not specified in the format string.
Either specify it in a placeholder or provide a default value with `#[sscanf(default)]` or `#[sscanf(default = ...)]`"));
        }
    }

    error.ok_or_build()?;

    // grab the type that each placeholder should be parsed to
    let mut types_by_placeholder = vec![None; format.placeholders.len()];
    for field in &fields {
        if let ValueSource::Placeholder { index, .. } = field.value_source {
            if types_by_placeholder[index].is_some() {
                // this case should never happen, because any place where we create ValueSource::Placeholder
                // we use a unique index and remove it from the map/iterator so that it can't be used again
                let ph = &format.placeholders[index];
                bail!(ph => "sscanf: Internal error: placeholder {index} is used multiple times");
            }
            types_by_placeholder[index] = Some(field.parsed_type.clone());
        }
    }

    // convert Vec<Option> to Option<Vec>
    let Some(types_by_placeholder): Option<Vec<_>> = types_by_placeholder.into_iter().collect()
    else {
        // this case should never happen, because we already checked through the unused placeholders
        bail!(attr.src => "sscanf: Internal error: A placeholder was not assigned a type");
    };

    let matcher = SequenceMatcher::new(&format, &types_by_placeholder, escape)?;

    let from_matches = fields.iter().map(|field| field.to_parser(&matcher.parsers));

    let parser = quote! { { #(#from_matches),* } };

    let mut lifetimes = HashSet::new();
    for field in &fields {
        if field.value_source.uses_parsed_type() {
            // if the field is parsed from a placeholder, we need to extract the lifetimes from the parsed type
            // and add them to the list of lifetimes
            extract_lifetimes(field.parsed_type.inner(), &mut lifetimes);
        }
    }

    Ok((matcher, parser, lifetimes))
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
            bail!(name => "at least one variant has to be constructable from sscanf and not skipped.");
        // checked in tests/fail/derive_enum_attributes.rs
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
