#![allow(
    dead_code,
    reason = "This is a utility module, some methods are here for completeness and in case they are needed in the future"
)]

use super::*;
use proc_macro2::Span;
use std::fmt::Display;

macro_rules! bail {
    ( $( { $span:expr => $format:expr $(, $arg:expr)* }, )+ ) => {{
        let mut build = ErrorBuilder::new();
        $(
            build.push($span.error(format_args!($format, $($arg),*)));
        )+
        return build.build_err();
    }};
    ( $span:expr => $format:expr $(, $arg:expr)* ) => {
        return Err($span.error(format_args!($format, $($arg),*)));
    };
}
macro_rules! assert_or_bail {
    ( $condition:expr, $span:expr => $message:expr $(, $arg:expr)* ) => {
        if !$condition {
            bail!($span => $message $(, $arg)*);
        }
    };
}
pub(crate) use {assert_or_bail, bail};

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

pub trait ToTokensErrExt {
    fn error(&self, message: impl Display) -> Error;
}
impl<S: quote::ToTokens> ToTokensErrExt for S {
    fn error(&self, message: impl Display) -> Error {
        Error::new_spanned(self, message)
    }
}

pub trait SpanErrExt {
    fn error(self, message: impl Display) -> Error;
}
impl SpanErrExt for Span {
    fn error(self, message: impl Display) -> Error {
        Error::new(self, message)
    }
}
