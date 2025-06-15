#![allow(unused)]

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::fmt::Display;

/// A type for wrapping one or more compile-time errors in a proc macro
pub struct Error(TokenStream);

/// A type alias for a result in a proc macro
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Creates a new error with the given span and message.
    pub fn new<T: Display>(span: Span, message: T) -> Self {
        syn::Error::new(span, message).into()
    }

    /// Creates a new error spanning the given tokens with the provided message.
    pub fn new_spanned<T: ToTokens, U: Display>(tokens: T, message: U) -> Self {
        syn::Error::new_spanned(tokens, message).into()
    }

    /// Shorthand for `Err(Error::new(span, message))`.
    ///
    /// Side Note: Honestly, this should be a general convenience thing somewhere in the Result world, because
    /// what else are you going to do with an error except wrap it in a `Result::Err`? And having to write
    /// `Err(Error...)` all the time is a bit redundant (and doesn't format too well if the error is longer).
    pub fn err<T: Display, R>(span: Span, message: T) -> Result<R> {
        Err(Self::new(span, message))
    }

    /// Shorthand for `Err(Error::new_spanned(tokens, message))`.
    pub fn err_spanned<T: ToTokens, U: Display, R>(tokens: T, message: U) -> Result<R> {
        Err(Self::new_spanned(tokens, message))
    }

    /// Creates a new error builder for combining multiple errors.
    pub fn builder() -> ErrorBuilder {
        ErrorBuilder::new()
    }
}

/// A builder for combining multiple errors into a single error.
pub struct ErrorBuilder(TokenStream);

impl ErrorBuilder {
    /// Creates a new empty error builder. Same as `Error::builder()`.
    fn new() -> Self {
        Self(TokenStream::new())
    }

    /// Adds a new error message to the builder with the given span.
    pub fn with<T: Display>(&mut self, span: Span, message: T) -> &mut Self {
        self.with_error(Error::new(span, message))
    }

    /// Adds a new error message to the builder spanning the given tokens.
    pub fn with_spanned<T: ToTokens, U: Display>(&mut self, tokens: T, message: U) -> &mut Self {
        self.with_error(Error::new_spanned(tokens, message))
    }

    /// Adds an existing error to the builder.
    pub fn with_error(&mut self, error: Error) -> &mut Self {
        self.0.extend(TokenStream::from(error));
        self
    }

    /// Adds an existing error to the builder. Same as `with_error`.
    pub fn push(&mut self, error: Error) {
        self.with_error(error);
    }

    /// Checks if the builder is empty, i.e. no errors have been added.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Builds the error from the collected messages and returns it.
    pub fn build(&mut self) -> Error {
        Error(std::mem::take(&mut self.0))
    }

    /// Builds the error and returns it as an `Err(Error)`.
    pub fn build_err<R>(&mut self) -> Result<R> {
        Err(self.build())
    }

    /// Checks if the builder is empty and returns `Ok(())` if it is, otherwise builds the error and returns it.
    ///
    /// This is useful for cases where you may or may not add errors and want to check it in a concise way:
    /// ```
    /// let mut error = Error::builder();
    ///
    /// // Code that may add errors
    ///
    /// error.ok_or_build()?; // Will early return if there was an error, does nothing otherwise
    /// ```
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
    /// Converts the error into a proc_macro2 `TokenStream`.
    fn from(err: Error) -> Self {
        err.0
    }
}

impl From<Error> for proc_macro::TokenStream {
    /// Converts the error into a proc_macro (1) `TokenStream`.
    fn from(err: Error) -> Self {
        err.0.into()
    }
}

/// Extension trait for `Result<TokenStream>` to convert it into a `TokenStream` or `proc_macro::TokenStream`.
pub trait TokenResultExt {
    /// Converts the result into a proc_macro2 `TokenStream`.
    fn into_token_stream(self) -> TokenStream;
    /// Converts the result into a proc_macro (1) `TokenStream`.
    fn into_proc_macro_token_stream(self) -> proc_macro::TokenStream;
}

impl TokenResultExt for Result<TokenStream> {
    fn into_token_stream(self) -> TokenStream {
        match self {
            Ok(ts) => ts,
            Err(err) => err.into(),
        }
    }

    fn into_proc_macro_token_stream(self) -> proc_macro::TokenStream {
        match self {
            Ok(ts) => ts.into(),
            Err(err) => err.into(),
        }
    }
}
