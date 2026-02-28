#![allow(
    dead_code,
    reason = "This is a utility module, some methods are here for completeness and in case they are needed in the future"
)]

use super::*;
use proc_macro2::Span;
pub use std::fmt::Display;

/// Macro to create and return an error.
///
/// The `$span:expr` arguments can be any type that implements the `ErrorTarget` or `ToTokensErrorTarget` trait.
macro_rules! bail {
    // macro arm for multiple spans with different messages
    // bail!(
    //     {span1 => msg1},
    //     {span2 => msg2}, // <-- trailing comma is required to differentiate macro arms
    //     ...
    // );
    ( $( { $span:expr => $format:literal $(, $arg:expr)* }, )+ ) => {{
        let mut build = ErrorBuilder::new();
        $(
            add_error!(build, $span => $format $(, $arg)*);
        )+
        return build.build_err();
    }};

    // macro arm for multiple spans with the same message
    // bail!({span1, span2, ...} => msg);
    ( { $($span:expr),* } => $format:literal $(, $arg:expr)* ) => {{
        let mut build = ErrorBuilder::new();
        let msg = format!($format $(, $arg)*);
        $(
            build.push($span.error(&msg));
        )+
        return build.build_err();
    }};

    // macro arm for a single span with a message
    // bail!(span => msg);
    ( $span:expr => $format:literal $(, $arg:expr)* ) => {
        return Err($span.error(format_args!($format $(, $arg)*)));
    };
}
/// Macro to assert a condition or return an error.
macro_rules! assert_or_bail {
    ( $condition:expr, $span:expr => $format:literal $(, $arg:expr)* ) => {
        if !$condition {
            bail!($span => $format $(, $arg)*);
        }
    };
}
/// Macro to add an error to an `ErrorBuilder`.
macro_rules! add_error {
    ( $error:ident, $span:expr => $format:literal $(, $arg:expr)* ) => {
        $error.push($span.error(format_args!($format $(, $arg)*)));
    };
}
pub(crate) use {add_error, assert_or_bail, bail};

pub struct ErrorBuilder(Option<Error>);

impl ErrorBuilder {
    pub fn new() -> Self {
        Self(None)
    }
    pub fn with<T: Display>(&mut self, span: Span, message: T) -> &mut Self {
        self.with_error(Error::new(span, message))
    }
    pub fn with_spanned<T: ToTokens, U: Display>(&mut self, tokens: T, message: U) -> &mut Self {
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
        self.0.take().unwrap() // it is up to the caller to ensure that there is an error
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

/// Trait for types that can be used as the part underlined in an error message.
pub trait ErrorTarget {
    /// Create an error from the source and message
    fn error(&self, message: impl Display) -> Error;
}

impl ErrorTarget for Span {
    fn error(&self, message: impl Display) -> Error {
        Error::new(*self, message)
    }
}

/// Like `ErrorTarget`, but for types that implement `ToTokens`.
///
/// Note that we don't just implement `ErrorTarget`, because the compiler will complain an upstream crate might
/// implement `ToTokens` for `Span`. (It won't, but the compiler can't know that.)
pub trait ToTokensErrorTarget {
    /// Create an error from the given tokens and message.
    fn error(&self, message: impl Display) -> Error;
}

impl<S: ToTokens> ToTokensErrorTarget for S {
    fn error(&self, message: impl Display) -> Error {
        Error::new_spanned(self, message)
    }
}

pub trait ResultExt {
    /// Convert the `Result` into a `TokenStream`, turning errors into compile errors.
    fn into_token_stream_1(self) -> proc_macro::TokenStream;
}
impl ResultExt for Result<TokenStream> {
    fn into_token_stream_1(self) -> proc_macro::TokenStream {
        match self {
            Ok(ts) => ts.into(),
            Err(e) => e.into_compile_error().into(),
        }
    }
}
