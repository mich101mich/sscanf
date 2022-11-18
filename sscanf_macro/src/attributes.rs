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
            return Err(syn::Error::new_spanned(
                eq_sign,
                "expected expression after `=`",
            ));
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
                error.with_spanned(&existing.src, format!("duplicate config option: {}", name));
                error.with_spanned(&option.src, format!("duplicate config option: {}", name));
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
}

impl From<AttributeArg> for DefaultAttribute {
    fn from(arg: AttributeArg) -> Self {
        Self {
            src: arg.src,
            value: arg.value,
        }
    }
}

impl ToTokens for DefaultAttribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(expr) = &self.value {
            expr.to_tokens(tokens);
        } else {
            tokens.extend(quote! { ::std::default::Default::default() });
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
                return Err(Error::new_spanned(
                    expr,
                    "expected closure expression for `map`",
                ))
            }
            None => {
                return Err(Error::new_spanned(
                    arg.src,
                    "expected closure expression for `map`",
                ))
            }
        };

        let param = match mapper.inputs.len() {
            1 => mapper.inputs.first().unwrap(),
            _ => {
                let msg = "expected `map` closure to take exactly one argument";
                return Err(Error::new_spanned(mapper, msg));
            }
        };

        let ty = if let syn::Pat::Type(ty) = param {
            (*ty.ty).clone()
        } else {
            let msg = "`map` closure has to specify the type of the argument";
            return Err(Error::new_spanned(param, msg));
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
