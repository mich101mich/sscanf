use crate::*;

pub struct StructAttributes {
    pub src: TokenStream,
    pub format: StrLit,
    pub escape: bool,
}

impl StructAttributes {
    pub fn new(attrs: Vec<syn::Attribute>) -> Result<Option<Self>> {
        let mut ret = None;
        let mut empty_attrs = None;
        for attr in attrs {
            if !attr.path.is_ident("sscanf") {
                continue;
            }
            if attr.parse_args::<TokenStream>()?.is_empty() {
                // trying to parse empty args the regular way would give
                // the Parse implementation no tokens to point an error to
                // => check for empty args here
                empty_attrs.get_or_insert(attr);
                continue;
            }
            let parsed = attr.parse_args::<Self>();
            match (ret.as_ref(), parsed) {
                (None, Ok(parsed)) => ret = Some(parsed),
                (None, Err(err)) => return Err(err.into()),
                (Some(prev), Ok(cur)) => {
                    let msg = "format attribute specified multiple times";
                    return Error::builder()
                        .with_spanned(&prev.src, msg)
                        .with_spanned(cur.src, msg)
                        .build_err(); // checked in tests/fail/derive_struct_attributes.rs
                }
                (Some(_), Err(_)) => {
                    let msg = "unneeded and invalid sscanf attribute";
                    return Error::err_spanned(attr, msg); // checked in tests/fail/derive_struct_attributes.rs
                }
            }
        }
        if let Some(ret) = ret {
            Ok(Some(ret))
        } else if let Some(attr) = empty_attrs {
            let msg = "expected attribute to take a format string as an argument.
Valid arguments: #[sscanf(format = \"...\")], #[sscanf(format_unescaped = \"...\")] or #[sscanf(\"...\")]";
            Error::err_spanned(attr, msg) // checked in tests/fail/derive_struct_attributes.rs
        } else {
            Ok(None)
        }
    }
}

impl Parse for StructAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src: TokenStream;
        let format: syn::LitStr;
        let mut escape = true;
        if input.peek(syn::LitStr) {
            format = input.parse()?;
            src = quote! { #format };
        } else if input.peek(syn::Ident) {
            let ident = input.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "format" => {}
                "format_unescaped" => {
                    escape = false;
                }
                "map" | "default" => {
                    let msg = format!("`{}` arguments are only valid on fields", ident);
                    return Err(syn::Error::new_spanned(ident, msg)); // checked in tests/fail/derive_struct_attributes.rs
                }
                s => {
                    let msg = if let Some(did_you_mean) =
                        find_similar(s, &["format", "format_unescaped"])
                    {
                        format!(
                            "unknown attribute `{}`. Did you mean `{}`?",
                            s, did_you_mean
                        )
                    } else {
                        format!(
                            "expected either `format` or `format_unescaped`, got `{}`",
                            s
                        )
                    };
                    return Err(syn::Error::new_spanned(ident, msg)); // checked in tests/fail/derive_struct_attributes.rs
                }
            }
            if input.is_empty() {
                let msg = format!("expected `= \"...\"` after `{}`", ident);
                return Err(syn::Error::new_spanned(ident, msg)); // checked in tests/fail/derive_struct_attributes.rs
            }
            let eq_sign = input.parse::<syn::Token![=]>()?;
            if input.is_empty() {
                let msg = "expected `\"...\"` after `=`";
                return Err(syn::Error::new_spanned(eq_sign, msg)); // checked in tests/fail/derive_struct_attributes.rs
            }
            format = input.parse()?;
            src = quote! { #ident #eq_sign #format };
        } else {
            let tokens = input.parse::<TokenStream>()?;
            let msg = "expected a format string as either `format = \"...\"`, `format_unescaped = \"...\"`, or just `\"...\"`";
            return Err(syn::Error::new_spanned(tokens, msg)); // checked in tests/fail/derive_struct_attributes.rs
        }

        let remaining = input.parse::<TokenStream>()?;
        if !remaining.is_empty() {
            let msg = "unnecessary arguments to the attribute. structs only allow a single `format` argument, nothing else";
            return Err(syn::Error::new_spanned(remaining, msg)); // checked in tests/fail/derive_struct_attributes.rs
        }

        Ok(StructAttributes {
            src,
            format: StrLit::new(format),
            escape,
        })
    }
}

pub struct FieldAttributes {
    pub src: TokenStream,
    pub kind: FieldAttributeKind,
}
pub enum FieldAttributeKind {
    Default(Option<syn::Expr>),
    Map {
        mapper: syn::ExprClosure,
        ty: syn::Type,
    },
}

impl FieldAttributes {
    pub fn new(attrs: Vec<syn::Attribute>) -> Result<Option<Self>> {
        let mut ret = None;
        for attr in attrs {
            if !attr.path.is_ident("sscanf") {
                continue;
            }
            if attr.tokens.is_empty() {
                continue;
            }
            if ret.is_some() {
                let msg = "fields can only have one `sscanf` attribute";
                return Error::err_spanned(attr, msg); // checked in tests/fail/derive_field_attributes.rs
            }
            ret = Some(attr.parse_args::<Self>()?);
        }
        Ok(ret)
    }
}

impl Parse for FieldAttributes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitStr) {
            let token = input.parse::<syn::LitStr>()?;
            let msg = "a string argument without a `<label> = ` prefix is only valid for format strings on structs and struct variants";
            return Err(syn::Error::new_spanned(token, msg)); // checked in tests/fail/derive_field_attributes.rs
        }

        let mut src = TokenStream::new();

        let ident = input.parse::<syn::Ident>()?;
        ident.to_tokens(&mut src);
        let kind = match ident.to_string().as_str() {
            "default" => {
                if input.is_empty() {
                    FieldAttributeKind::Default(None)
                } else {
                    let eq_sign = input.parse::<syn::Token![=]>()?;
                    eq_sign.to_tokens(&mut src);
                    if input.is_empty() {
                        let msg = "expected an expression after `=`";
                        return Err(syn::Error::new_spanned(eq_sign, msg)); // checked in tests/fail/derive_field_attributes.rs
                    }
                    let expr = input.parse::<syn::Expr>()?;
                    expr.to_tokens(&mut src);
                    FieldAttributeKind::Default(Some(expr))
                }
            }
            "map" => {
                if input.is_empty() {
                    let msg = "format for map attributes is `#[sscanf(map = |<arg>: <type>| <conversion>)]`";
                    return Err(syn::Error::new_spanned(ident, msg)); // checked in tests/fail/derive_field_attributes.rs
                }
                let eq_sign = input.parse::<syn::Token![=]>()?;
                eq_sign.to_tokens(&mut src);
                if input.is_empty() {
                    let msg = "map attribute expects a closure like `|<arg>: <type>| <conversion>` after `=`";
                    return Err(syn::Error::new_spanned(eq_sign, msg)); // checked in tests/fail/derive_field_attributes.rs
                }

                if !input.peek(Token![|]) {
                    let tokens = input.parse::<TokenStream>()?;
                    let msg = "map attribute expects a closure like `|<arg>: <type>| <conversion>` after `=`";
                    return Err(syn::Error::new_spanned(tokens, msg)); // checked in tests/fail/derive_field_attributes.rs
                }

                let mapper = input.parse::<syn::ExprClosure>()?;
                mapper.to_tokens(&mut src);

                let param = if mapper.inputs.len() == 1 {
                    mapper.inputs.first().unwrap()
                } else {
                    let msg = "expected `map` closure to take exactly one argument";
                    let mut span_src = TokenStream::new();
                    for param in mapper.inputs.pairs().skip(1) {
                        param.to_tokens(&mut span_src);
                    }
                    if span_src.is_empty() {
                        // no arguments were given => point to the empty `||`
                        mapper.or1_token.to_tokens(&mut span_src);
                        mapper.or2_token.to_tokens(&mut span_src);
                    }
                    return Err(syn::Error::new_spanned(span_src, msg)); // checked in tests/fail/derive_field_attributes.rs
                };

                let ty = if let syn::Pat::Type(ty) = param {
                    (*ty.ty).clone()
                } else {
                    let msg = "`map` closure has to specify the type of the argument";
                    return Err(syn::Error::new_spanned(param, msg)); // checked in tests/fail/derive_field_attributes.rs
                };

                FieldAttributeKind::Map { mapper, ty }
            }
            "format" | "format_unescaped" => {
                let msg = "format strings can only be specified on structs and struct variants, not fields";
                return Err(syn::Error::new_spanned(ident, msg)); // checked in tests/fail/derive_field_attributes.rs
            }
            s => {
                let msg = if let Some(did_you_mean) = find_similar(s, &["default", "map"]) {
                    format!(
                        "unknown attribute `{}`. Did you mean `{}`?",
                        s, did_you_mean
                    )
                } else {
                    format!("expected either `default` or `map`, got `{}`", s)
                };
                return Err(syn::Error::new_spanned(ident, msg)); // checked in tests/fail/derive_field_attributes.rs
            }
        };

        let remaining = input.parse::<TokenStream>()?;
        if !remaining.is_empty() {
            let msg = "unnecessary arguments to the attribute. fields only allow a single attribute";
            return Err(syn::Error::new_spanned(remaining, msg)); // checked in tests/fail/derive_field_attributes.rs
        }

        Ok(FieldAttributes { src, kind })
    }
}

fn find_similar<'a>(s: &str, compare: &[&'a str]) -> Option<&'a str> {
    let mut best_confidence = 0.0;
    let mut best_match = None;
    for valid in compare {
        let confidence = strsim::jaro_winkler(s, valid);
        if confidence > best_confidence {
            best_confidence = confidence;
            best_match = Some(*valid);
        }
    }
    if best_confidence > 0.8 {
        best_match
    } else {
        None
    }
}
