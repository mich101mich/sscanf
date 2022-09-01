use std::collections::HashMap;

use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    parse2, Attribute, ExprAssign, Ident,
};

use crate::*;

pub struct FormatAttribute {
    pub name: Ident,
    pub value: StrLit,
    pub src: ExprAssign,
}
impl Parse for FormatAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let src = input.parse::<ExprAssign>()?;
        let name = parse2(src.left.to_token_stream())?;
        let value = parse2(src.right.to_token_stream())?;
        Ok(FormatAttribute { src, name, value })
    }
}
impl FormatAttribute {
    pub fn from_attrs(attr: &Attribute) -> Result<HashMap<String, FormatAttribute>> {
        struct VecWrapper(Vec<FormatAttribute>);
        impl Parse for VecWrapper {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let content = input
                    .parse_terminated::<_, Token![,]>(FormatAttribute::parse)?
                    .into_iter()
                    .collect();
                Ok(VecWrapper(content))
            }
        }
        let list = attr.parse_args::<VecWrapper>()?.0;
        let mut ret = HashMap::<String, FormatAttribute>::new();
        for option in list {
            let name = option.name.to_string();
            if let Some(existing) = ret.get(&name) {
                return Error::builder()
                    .with_spanned(&existing.src, format!("duplicate config option: {}", name))
                    .with_spanned(&option.src, format!("duplicate config option: {}", name))
                    .build_err();
            }
            ret.insert(name, option);
        }
        Ok(ret)
    }
}

pub struct DefaultAttribute(pub TokenStream);
impl Parse for DefaultAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;

        if ident != "default" {
            return Err(syn::Error::new_spanned(ident, "expected `default`"));
        }

        if input.is_empty() {
            let span = ident.span();
            let value = quote_spanned!(span => ::std::default::Default::default());
            return Ok(DefaultAttribute(value));
        }

        let _ = input.parse::<Token![=]>()?;
        let value = input.parse::<Expr>()?;

        if !input.is_empty() {
            let remaining = input.parse::<TokenStream>()?;
            let msg = "unexpected tokens after `default`";
            return Err(syn::Error::new_spanned(remaining, msg));
        }

        Ok(DefaultAttribute(value.to_token_stream()))
    }
}
