//! Advanced parsing types and utilities.

pub(crate) mod format_options;
pub(crate) mod match_tree;
pub(crate) mod matcher;
pub use format_options::*;
pub use match_tree::*;
pub use matcher::*;

/// Marks types that support regex overrides in format strings.
///
/// Implement this trait to allow a regex override in the format string, e.g. `{MyType:/my-regex/}`.
///
/// Complex types (e.g., structs) rely on specific capture groups to parse fields, so overriding the regex is outside
/// the scope of `FromScanf`.
#[diagnostic::on_unimplemented(
    message = "type `{Self}` doesn't support regex overrides in `sscanf!`",
    label = "can't use regex override for this type",
    note = "Regex overrides are mostly only available for simple types. For more complex types, consider implementing `FromScanf` for a wrapper.",
    note = "See the `AcceptsRegexOverride` documentation for details: <https://docs.rs/sscanf/latest/sscanf/advanced/trait.AcceptsRegexOverride.html>"
)]
pub trait AcceptsRegexOverride<'input>: Sized {
    /// Callback to parse the input string from a regex match.
    ///
    /// Only the full match is passed to this function, not individual capture groups. A regex override can change the
    /// number and meaning of capture groups, so the type cannot rely on them.
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
    /// Provide a custom implementation when the type uses format options (e.g., number parsing).
    fn from_regex_match(input: &'input str, format: &FormatOptions) -> Option<Self>;
}
