use std::collections::HashMap;

use quote::ToTokens;

use crate::*;

pub struct AttributeArg {
    pub src: TokenStream,
    pub name: syn::Ident,
    pub value: Option<syn::Expr>,
}

pub type AttributeArgMap = HashMap<String, AttributeArg>;

impl Parse for AttributeArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<syn::Ident>()?;

        if input.is_empty() || input.peek(syn::Token![,]) {
            return Ok(Self {
                src: name.to_token_stream(),
                name,
                value: None,
            });
        }

        let eq_sign = input.parse::<Token![=]>()?;

        if input.is_empty() || input.peek(syn::Token![,]) {
            let msg = "expected expression after `=`";
            return Err(syn::Error::new_spanned(eq_sign, msg)); // checked in tests/fail/derive_field_attributes.rs
        }

        let value = input.parse::<Expr>()?;

        Ok(Self {
            src: quote! { #name #eq_sign #value },
            name,
            value: Some(value),
        })
    }
}

impl AttributeArg {
    pub fn from_attrs(attr: &syn::Attribute) -> Result<AttributeArgMap> {
        struct VecWrapper(Vec<AttributeArg>);
        impl Parse for VecWrapper {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let content = input
                    .parse_terminated::<_, Token![,]>(AttributeArg::parse)?
                    .into_iter()
                    .collect();
                Ok(VecWrapper(content))
            }
        }
        let list = attr.parse_args::<VecWrapper>()?.0;
        let mut ret = AttributeArgMap::new();
        let mut error = Error::builder();
        for option in list {
            let name = option.name.to_string();
            if let Some(existing) = ret.get(&name) {
                let msg = format!("duplicate attribute arg: {}", name);
                error.with_spanned(&existing.src, &msg); // checked in tests/fail/derive_field_attributes.rs
                error.with_spanned(&option.src, &msg); // checked in tests/fail/derive_field_attributes.rs
            }
            ret.insert(name, option);
        }
        error.ok_or_build()?;
        Ok(ret)
    }
}

pub struct DefaultAttribute {
    pub src: TokenStream,
    pub value: Option<Expr>,
    ty_span: FullSpan,
}

impl DefaultAttribute {
    pub fn new(arg: AttributeArg, ty: &syn::Type) -> Self {
        Self {
            src: arg.src,
            value: arg.value,
            ty_span: FullSpan::from_spanned(ty),
        }
    }
}

impl ToTokens for DefaultAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(expr) = &self.value {
            expr.to_tokens(tokens);
        } else {
            self.ty_span
                .apply(quote! { ::std::default::Default }, quote! { ::default() })
                .to_tokens(tokens);
        }
    }
}

pub struct MapperAttribute {
    pub src: TokenStream,
    pub mapper: syn::ExprClosure,
    pub ty: syn::Type,
}

impl std::convert::TryFrom<AttributeArg> for MapperAttribute {
    type Error = Error;
    fn try_from(arg: AttributeArg) -> Result<Self> {
        let mapper = match arg.value {
            Some(Expr::Closure(closure)) => closure,
            Some(expr) => {
                let msg = "expected closure expression for `map`";
                return Error::err_spanned(expr, msg); // checked in tests/fail/derive_field_attributes.rs
            }
            None => {
                let msg = "expected closure expression for `map`";
                return Error::err_spanned(arg.src, msg); // checked in tests/fail/derive_field_attributes.rs
            }
        };

        let param = match mapper.inputs.len() {
            1 => mapper.inputs.first().unwrap(),
            0 => {
                let msg = "expected `map` closure to take exactly one argument";
                let mut span_src = mapper.or1_token.to_token_stream();
                mapper.or2_token.to_tokens(&mut span_src);
                return Error::err_spanned(span_src, msg); // checked in tests/fail/derive_field_attributes.rs
            }
            _ => {
                let msg = "expected `map` closure to take exactly one argument";
                let mut span_src = TokenStream::new();
                for param in mapper.inputs.pairs().skip(1) {
                    param.to_tokens(&mut span_src);
                }
                return Error::err_spanned(span_src, msg); // checked in tests/fail/derive_field_attributes.rs
            }
        };

        let ty = if let syn::Pat::Type(ty) = param {
            (*ty.ty).clone()
        } else {
            let msg = "`map` closure has to specify the type of the argument";
            return Error::err_spanned(param, msg); // checked in tests/fail/derive_field_attributes.rs
        };

        Ok(Self {
            src: arg.src,
            mapper,
            ty,
        })
    }
}

impl ToTokens for MapperAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.mapper.to_tokens(tokens);
    }
}
