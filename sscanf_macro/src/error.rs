#![allow(
    dead_code,
    reason = "This is a utility module, some methods are here for completeness and in case they are needed in the future"
)]

use super::*;
use proc_macro2::Span;
pub use std::fmt::Display;

macro_rules! bail {
    // macro arm for multiple spans with different messages
    // bail!(
    //     {span1 => msg1},
    //     {span2 => msg2}, // <-- trailing comma is required to differentiate macro arms
    //     ...
    // );
    ( $( { $span:expr => $format:expr $(, $arg:expr)* }, )+ ) => {{
        let mut build = ErrorBuilder::new();
        $(
            build.push(error!($span => $format $(, $arg)*));
        )+
        return build.build_err();
    }};

    // macro arm for multiple spans with the same message
    // bail!({span1, span2, ...} => msg);
    ( { $($span:expr),* } => $format:expr $(, $arg:expr)* ) => {{
        let mut build = ErrorBuilder::new();
        let msg = format!($format, $($arg),*);
        $(
            build.push($span.error(&msg));
        )+
        return build.build_err();
    }};

    // macro arm for a single span with a message
    // bail!(span => msg);
    ( $span:expr => $format:expr $(, $arg:expr)* ) => {
        return Err(error!($span => $format $(, $arg)*));
    };
}
macro_rules! assert_or_bail {
    ( $condition:expr, $span:expr => $message:expr $(, $arg:expr)* ) => {
        if !$condition {
            bail!($span => $message $(, $arg)*);
        }
    };
}
macro_rules! error {
    ( $span:expr => $format:expr $(, $arg:expr)* ) => {
        $span.error(format_args!($format, $($arg),*))
    };
}
pub(crate) use {assert_or_bail, bail, error};

pub struct ErrorBuilder(Option<Error>);

impl ErrorBuilder {
    pub fn new() -> Self {
        Self(None)
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
        match self.0 {
            Some(ref mut existing) => existing.combine(error),
            None => self.0 = Some(error),
        }
        self
    }
    pub fn push(&mut self, error: Error) {
        self.with_error(error);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn build(&mut self) -> Error {
        self.0.take().unwrap()
    }
    pub fn build_err<R>(&mut self) -> Result<R> {
        Err(self.build())
    }
    pub fn ok_or_build(&mut self) -> Result<()> {
        if let Some(err) = self.0.take() {
            Err(err)
        } else {
            Ok(())
        }
    }
}

/// Extension trait for anything that can be converted to tokens to create errors
pub trait ToTokensErrExt {
    /// Create an error from the given tokens and message
    fn error(&self, message: impl Display) -> Error;
}
impl<S: quote::ToTokens> ToTokensErrExt for S {
    fn error(&self, message: impl Display) -> Error {
        Error::new_spanned(self, message)
    }
}

/// Extension trait for spans to create errors
pub trait SpanErrExt {
    /// Create an error from the span and message
    fn error(self, message: impl Display) -> Error;
}
impl SpanErrExt for Span {
    fn error(self, message: impl Display) -> Error {
        Error::new(self, message)
    }
}

/// Trait for types that have a source, used for error reporting
pub trait Sourced<'a> {
    /// Create an error from the source and message
    fn error(&self, message: impl Display) -> Error;
}
