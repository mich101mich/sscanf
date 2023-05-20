use proc_macro2::{Span, TokenStream};
use std::fmt::Display;

pub struct Error(TokenStream);

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new<T: Display>(span: Span, message: T) -> Self {
        syn::Error::new(span, message).into()
    }
    pub fn new_spanned<T: quote::ToTokens, U: Display>(tokens: T, message: U) -> Self {
        syn::Error::new_spanned(tokens, message).into()
    }
    pub fn err<T: Display, R>(span: Span, message: T) -> Result<R> {
        Err(Self::new(span, message))
    }
    pub fn err_spanned<T: quote::ToTokens, U: Display, R>(tokens: T, message: U) -> Result<R> {
        Err(Self::new_spanned(tokens, message))
    }
    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }
}

pub struct ErrorBuilder(TokenStream);

impl ErrorBuilder {
    fn new() -> Self {
        Self(TokenStream::new())
    }
    pub fn with<T: Display>(&mut self, span: Span, message: T) -> &mut Self {
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
        self.0.extend(TokenStream::from(error));
        self
    }
    pub fn push(&mut self, error: Error) {
        self.with_error(error);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn build(&mut self) -> Error {
        Error(std::mem::take(&mut self.0))
    }
    pub fn build_err<R>(&mut self) -> Result<R> {
        Err(self.build())
    }
    pub fn ok_or_build(&mut self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            self.build_err()
        }
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Error(err.to_compile_error())
    }
}

impl From<TokenStream> for Error {
    fn from(err: TokenStream) -> Self {
        Error(err)
    }
}

impl From<Error> for TokenStream {
    fn from(err: Error) -> Self {
        err.0
    }
}
impl From<Error> for proc_macro::TokenStream {
    fn from(err: Error) -> Self {
        err.0.into()
    }
}
