use crate::*;

#[derive(Clone)]
pub struct Type<'a> {
    pub source: TypeSource<'a>,
    ty: syn::Type,
}

#[derive(Clone)]
pub enum TypeSource<'a> {
    External,
    Format(StrLitSlice<'a>),
}

#[allow(unused)]
impl<'a> Type<'a> {
    pub fn from_ty(ty: syn::Type) -> Self {
        let source = TypeSource::External;
        Type { source, ty }
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
    pub fn from_str(src: StrLitSlice<'a>) -> syn::Result<Self> {
        let span = src.span();

        let tokens = src.text().parse::<TokenStream>()?.with_span(span);
        let mut ty = syn::parse2::<Type>(tokens)?;
        ty.source = TypeSource::Format(src);
        Ok(ty)
    }
}

impl Parse for Type<'_> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: syn::Type = if input.peek(Token![&]) {
            // possibly &str
            input.parse::<syn::TypeReference>()?.into()
        } else {
            input.parse::<syn::TypePath>()?.into()
        };
        Ok(Self::from_ty(ty))
    }
}

impl quote::ToTokens for Type<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ty.to_tokens(tokens);
    }
}
