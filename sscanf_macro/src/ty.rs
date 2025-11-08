use crate::*;

#[derive(Clone)]
pub struct Type<'a> {
    pub source: TypeSource<'a>,
    pub field_name: Option<String>,
    ty: syn::Type,
}

#[derive(Clone)]
pub enum TypeSource<'a> {
    External,
    Format(StrLitSlice<'a>),
}

#[allow(
    dead_code,
    reason = "Some methods are here fore completeness and in case they are needed in the future"
)]
impl<'a> Type<'a> {
    pub fn from_ty(mut ty: syn::Type) -> Self {
        let source = TypeSource::External;

        if let syn::Type::Path(syn::TypePath { qself: None, path }) = &ty
            && path.is_ident("str")
        {
            // str used to be hardcoded to take "str" as input and return a `&str` type.
            // Since &str now directly implements `FromScanf`, we no longer need the workaround,
            // but we want to keep the ability to just use `{str}` in the format string.
            ty = syn::Type::Reference(syn::TypeReference {
                and_token: Token![&](ty.span()),
                lifetime: None,
                mutability: None,
                elem: Box::new(ty),
            });
        }

        Type {
            source,
            field_name: None,
            ty,
        }
    }

    pub fn from_field(ty: syn::Type, field_name: String) -> Self {
        let mut ret = Self::from_ty(ty);
        ret.field_name = Some(field_name);
        ret
    }

    pub fn inner(&self) -> &syn::Type {
        &self.ty
    }
    pub fn into_inner(self) -> syn::Type {
        self.ty
    }
    pub fn full_span(&self) -> FullSpan {
        match &self.source {
            TypeSource::External => FullSpan::from_spanned(&self.ty),
            TypeSource::Format(src) => FullSpan::from_span(src.span()),
        }
    }

    pub fn err<T, U: std::fmt::Display>(&self, message: U) -> Result<T> {
        Err(self.error(message))
    }
    pub fn error<U: std::fmt::Display>(&self, message: U) -> Error {
        match &self.source {
            TypeSource::External => Error::new_spanned(&self.ty, message),
            TypeSource::Format(src) => src.error(message),
        }
    }
    pub fn from_str(src: StrLitSlice<'a>) -> Result<Self> {
        let span = src.span();

        let tokens = src.text().parse::<TokenStream>()?.with_span(span);
        let mut ty = syn::parse2::<Type>(tokens)?;
        ty.source = TypeSource::Format(src);
        Ok(ty)
    }
}

impl Parse for Type<'_> {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the input as a syn::Type.
        // Note that we don't directly parse syn::Type because there are far too many possible variations of types,
        // like `fn()`, `impl Trait`, `_`, `[T]`, etc.
        // The vast majority of these are not valid in sscanf, and they would just clutter the
        // error message, since syn always says "expected one of fn, impl, _, [, ..." with all possible

        // let ty = if input.peek(Token![&]) {
        //     // possibly &str
        //     input.parse::<syn::TypeReference>()?.into()
        // } else {
        //     input.parse::<syn::TypePath>()?.into()
        // };
        Ok(Self::from_ty(input.parse::<syn::Type>()?))
    }
}

impl quote::ToTokens for Type<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens);
    }
}
