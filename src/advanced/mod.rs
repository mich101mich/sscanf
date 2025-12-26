//! Types and utilities for advanced FromScanf parsing

pub(crate) mod format_options;
pub(crate) mod match_tree;
pub(crate) mod matcher;
pub(crate) mod parser;
pub use format_options::*;
pub use match_tree::*;
pub use matcher::*;
pub use parser::*;

/// Extra trait that needs to be implemented for types that can accept a regex override in the format string
///
/// This trait needs to be implemented for a type in order to have a regex override in the format string like
/// `{MyType:/my-regex/}`.
///
/// This is not handled by the core `FromScanf` trait, since complex types like structs rely on specific capture groups
/// to parse their fields, and a regex override would break that assumption.
#[diagnostic::on_unimplemented(
    message = "type `{Self}` doesn't support regex overrides in `sscanf!`",
    label = "can't use regex override for this type",
    note = "Regex overrides are mostly only available for simple types. For more complex types, consider implementing `FromScanf` for a wrapper.",
    note = "See the `AcceptsRegexOverride` documentation for details: <https://docs.rs/sscanf/latest/sscanf/advanced/trait.AcceptsRegexOverride.html>"
)]
pub trait AcceptsRegexOverride<'input>: Sized {
    /// Callback to parse the input string from a regex match.
    ///
    /// Note that only the full match is passed to this function, not the individual capture groups.
    /// This is because the regex override can change the number and meaning of capture groups, meaning
    /// the type can't rely on them.
    ///
    /// For most types, this can simply fall back to [`FromStr`](std::str::FromStr), since it just needs to parse
    /// the type from a string:
    /// ```rust
    /// use sscanf::advanced::{AcceptsRegexOverride, FormatOptions};
    ///
    /// struct TypeWithFromStr { /* fields */ }
    ///
    /// impl std::str::FromStr for TypeWithFromStr {
    ///     // ... implementation ...
    ///     # type Err = ();
    ///     # fn from_str(_s: &str) -> Result<Self, Self::Err> { Ok(TypeWithFromStr {}) }
    /// }
    ///
    /// impl AcceptsRegexOverride<'_> for TypeWithFromStr {
    ///    fn from_regex_match(input: &str, _format: &FormatOptions) -> Option<Self> {
    ///        input.parse().ok()
    ///    }
    /// }
    /// ```
    ///
    /// The main reason to provide a custom implementation is if the type accepts format options for e.g. number
    /// parsing.
    fn from_regex_match(input: &'input str, format: &FormatOptions) -> Option<Self>;
}
