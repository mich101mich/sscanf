use std::fmt::Display;

pub enum Error {
    Basic(syn::Error),
    Custom(proc_macro2::TokenStream),
}
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new<T: Display>(span: proc_macro2::Span, message: T) -> Self {
        Error::Basic(syn::Error::new(span, message))
    }
    pub fn new_spanned<T: quote::ToTokens, U: Display>(tokens: T, message: U) -> Self {
        Error::Basic(syn::Error::new_spanned(tokens, message))
    }
    pub fn err<T: Display, R>(span: proc_macro2::Span, message: T) -> Result<R> {
        Err(Error::new(span, message))
    }
    pub fn err_spanned<T: quote::ToTokens, U: Display, R>(tokens: T, message: U) -> Result<R> {
        Err(Error::new_spanned(tokens, message))
    }
    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }
    pub fn to_compile_error(self) -> proc_macro2::TokenStream {
        match self {
            Error::Basic(err) => err.to_compile_error(),
            Error::Custom(tokens) => tokens,
        }
    }
}

pub struct ErrorBuilder {
    tokens: proc_macro2::TokenStream,
}
impl ErrorBuilder {
    fn new() -> Self {
        ErrorBuilder {
            tokens: proc_macro2::TokenStream::new(),
        }
    }
    pub fn with<T: Display>(&mut self, span: proc_macro2::Span, message: T) -> &mut Self {
        self.with_error(Error::new(span, message))
    }
    pub fn with_spanned<T: quote::ToTokens, U: Display>(
        &mut self,
        tokens: T,
        message: U,
    ) -> &mut Self {
        self.with_error(Error::new_spanned(tokens, message))
    }
    pub fn with_error(&mut self, error: Error) -> &mut Self {
        self.tokens.extend(error.to_compile_error());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn build(&mut self) -> Error {
        Error::Custom(std::mem::replace(
            &mut self.tokens,
            proc_macro2::TokenStream::new(),
        ))
    }
    pub fn build_err<R>(&mut self) -> Result<R> {
        Err(self.build())
    }
    pub fn ok_or_build(&mut self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self.build())
        }
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Error::Basic(err)
    }
}

impl From<proc_macro2::TokenStream> for Error {
    fn from(err: proc_macro2::TokenStream) -> Self {
        Error::Custom(err)
    }
}

impl From<Error> for proc_macro2::TokenStream {
    fn from(err: Error) -> Self {
        err.to_compile_error()
    }
}
impl From<Error> for proc_macro::TokenStream {
    fn from(err: Error) -> Self {
        err.to_compile_error().into()
    }
}
