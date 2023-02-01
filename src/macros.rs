//! A file with the macro re-exports to separate the documentation from the crate root docs

/// A Macro to parse a string based on a format-string, similar to sscanf in C
///
/// ## Signature
/// ```ignore
/// sscanf!(input: impl Deref<Target=str>, format: <literal>, Type...) -> Result<(Type...), sscanf::Error>
/// ```
///
/// ## Parameters
/// * `input`: The string to parse. Can be anything that implements [`Deref<Target=str>`](std::ops::Deref)
///   (e.g. `&str`, `String`, `Cow<str>`, etc. See examples below). Note that `sscanf` does not take
///   ownership of the input.
/// * `format`: A literal string. No const or static allowed, just like with [`format!()`](std::format).
/// * `Type...`: The types to parse. See [Custom Types](index.html#custom-types) for more information.
///
/// ## Return Value
/// A [`Result`](std::result::Result) with a tuple of the parsed types or a [`sscanf::Error`](crate::errors::Error).
/// Note that an error usually indicates that the input didn't match the format string, making the
/// returned [`Result`](std::result::Result) functionally equivalent to an [`Option`](std::option::Option),
/// and most applications should treat it that way. An error is only useful when debugging
/// custom implementations of [`FromStr`](std::str::FromStr) or [`FromScanf`](crate::FromScanf).
/// See [`sscanf::Error`](crate::errors::Error) for more information.
///
/// ## Details
/// The format string _has_ to be a string literal (with some form of `"` on either side),
/// because it is parsed by the procedural macro at compile time and checks if all the types
/// and placeholders are matched. This is not possible from inside a variable or even a `const
/// &str` somewhere else.
///
/// Placeholders within the format string are marked with `{}`. Any `'{'` or `'}'` that should not be
/// treated as placeholders need to be escaped by writing `'{{'` or `'}}'`. For every placeholder there
/// has to be a type name inside the `{}` or exactly one type in the parameters after the format
/// string. Types can be referenced by indices in the placeholder, similar to [`format!()`](std::fmt).
///
/// Any additional formatting options are placed behind a `:`. For a list of options, see
/// the [crate root documentation](index.html#format-options).
///
/// ## Examples
/// A few examples for possible inputs:
/// ```
/// # use sscanf::sscanf;
/// let input = "5"; // &str
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input = String::from("5"); // String
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input = &input; // &String
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
/// assert_eq!(sscanf!(input.as_str(), "{usize}").unwrap(), 5);
///
/// let input = std::borrow::Cow::from("5"); // Cow<str>
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input = std::rc::Rc::from("5"); // Rc<str>
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// // and many more
/// ```
///
/// ```compile_fail
/// // temporary value: doesn't work
/// sscanf!(String::from("5"), "{usize}");
/// ```
///
/// More Examples can be seen in the crate root documentation.
pub use sscanf_macro::sscanf;

#[doc(hidden)]
pub use sscanf_macro::sscanf as scanf;

/// Same as [`sscanf`], but returns the regex without running it. Useful for debugging or efficiency.
///
/// ## Signature
/// ```ignore
/// sscanf_get_regex!(format: <literal>, Type...) -> &'static Regex
/// ```
///
/// ## Parameters
/// * `format`: A literal string. No const or static allowed, just like with [`format!()`](std::format).
/// * `Type...`: The types to parse. See [Custom Types](index.html#custom-types) for more information.
///
/// Returns: A reference to the generated [`Regex`](regex::Regex).
///
/// The Placeholders can be obtained by capturing the Regex and using the 1-based index of the Group.
///
/// ## Examples
/// ```
/// use sscanf::sscanf_get_regex;
/// let input = "Test 5 -2";
/// let regex = sscanf_get_regex!("Test {usize} {i32}");
/// assert_eq!(regex.as_str(), r"^Test (\+?\d{1,20}) ([-+]?\d{1,10})$");
///
/// let output = regex.captures(input);
/// assert!(output.is_some());
/// let output = output.unwrap();
///
/// let capture_5 = output.get(1);
/// assert!(capture_5.is_some());
/// assert_eq!(capture_5.unwrap().as_str(), "5");
///
/// let capture_negative_2 = output.get(2);
/// assert!(capture_negative_2.is_some());
/// assert_eq!(capture_negative_2.unwrap().as_str(), "-2");
/// ```
pub use sscanf_macro::sscanf_get_regex;

#[doc(hidden)]
pub use sscanf_macro::sscanf_get_regex as scanf_get_regex;

/// Same as [`sscanf`], but allows use of Regex in the format String.
///
/// Signature and Parameters are the same as [`sscanf`].
///
/// ## Examples
/// ```
/// use sscanf::sscanf_unescaped;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = sscanf_unescaped!(input, "{f32}.*?{usize}"); // .*? matches anything
/// assert_eq!(output.unwrap(), (5.0, 3));
/// ```
///
/// The basic [`sscanf`] would escape the `.`, `*` and `?`and match against the literal Characters,
/// as one would expect from a Text matcher:
/// ```
/// use sscanf::sscanf;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = sscanf!(input, "{f32}.*{usize}");
/// assert!(output.is_err()); // does not match
///
/// let input2 = "5.0.*3";
/// let output2 = sscanf!(input2, "{f32}.*{usize}"); // regular sscanf is unaffected by special characters
/// assert_eq!(output2.unwrap(), (5.0, 3));
/// ```
///
/// Note that the `{{` and `}}` escaping for literal `{` and `}` is still required.
///
/// Also note that `^` and `$` are automatically added to the start and end.
pub use sscanf_macro::sscanf_unescaped;

#[doc(hidden)]
pub use sscanf_macro::sscanf_unescaped as scanf_unescaped;

/// A derive macro for [`FromScanf`](crate::FromScanf).
///
/// ## For structs
/// ```ignore
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "<format>")] // format string. has to contain placeholders for all
/// struct MyStruct {              // non-default fields: {<field>}, {<field_2>}, {<field_with_conversion>}
///
///     <field>: <type>, // requires <type>: FromScanf (implemented for all primitive types
///                      // and several others from std)
///
///     <field_2>: <type_2>, // requires <type_2>: FromScanf
///
///     // ...
///
///     // possible attributes on fields:
///
///     #[sscanf(default)]
///     <field_with_default>: <type>, // requires <type>: Default, but doesn't need FromScanf
///
///     #[sscanf(default = <expression>)] // accepts any expression that returns <type>
///     <field_with_custom_default>: <type>, // no traits required.
///
///     #[sscanf(map = |input: <matched_type>| { <conversion from <matched_type> to <actual_type>> })]
///     <field_with_conversion>: <actual_type>, // requires <matched_type>: FromScanf
/// }
///
/// // tuple structs have the same capabilities, just without field names:
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "<format>")] // format string references fields by index: {0}, ...
/// struct MyTupleStruct(<type>, #[sscanf(default)] <type>, ...);
/// ```
///
/// **\<format>**: The format string to parse the struct from. Similar to the format string for
/// [`sscanf`], but with field names/indices instead of types for the placeholders. So, if you have
/// a struct with fields `a`, `b` and `c`, the format string could be something like
/// `"{a} {b:/.*?/} {c}"`. All fields that are not annotated with `default` must appear exactly
/// once in the format string. Indices can be omitted if the fields are in the same order as the
/// placeholders `{}` in the format string. So, the above example could also be written as
/// `"{} {:/.*?/} {}"`.
///
/// Any fields that don't appear in the format string must be annotated with `default`. The field
/// will then be initialized to either [`Default::default()`](std::default::Default) or the evaluation of the expression
/// given to the `default` attribute. The expression can be any code, including function calls or
/// `{ <code> }` blocks, as long as they can be assigned to the field type. In the syntax overview
/// above, the `<format>` must contain `<field>`, `<field_2>` and `<field_with_conversion>`
/// exactly once and neither `<field_with_default>` nor `<field_with_custom_default>` must appear
/// in the format string.
///
/// If the format string has to contain raw regex (like with [`sscanf_unescaped`]), write
/// `format_unescaped = r"..."` or just `r"..."` instead of `format = "..."` or `"..."`.
///
/// Mapping allows matching against a different type than the field type. The `map` attribute takes
/// a closure that takes the matched type as input and returns the field type. The type of the
/// parameter of the closure has to be explicitly specified, since it is needed to generate the
/// matching code.
///
/// The types of the used fields of their matching types have to implement [`FromScanf`](crate::FromScanf)
/// and either [`RegexRepresentation`](crate::RegexRepresentation) or have a `{<field>:/<regex>/}`
/// placeholder.
///
/// ## For enums
/// ```ignore
/// #[derive(sscanf::FromScanf)]
/// enum MyEnum {
///     #[sscanf(format = "<format>")] // has to contain `{<field>}` and any other fields
///     Variant1 {
///         <field>: <type>, // requires <type>: FromScanf
///
///         #[sscanf(default)]
///         <field_with_default>: <type2>, // requires <type2>: Default
///
///         // ... (same as for structs)
///     },
///
///     #[sscanf("<format>")] // the `format = ` part can be omitted
///     Variant2(<type>, #[sscanf(default)] <type2>),
///
///     #[sscanf("<format>")] // variant has no fields => no placeholders in format string
///     Variant3,
///
///     Variant4, // this variant won't be constructed by sscanf
/// }
/// ```
/// An enum takes multiple format strings, one for each variant. The value returned from `sscanf`
/// is constructed from the variant that matched the input. If multiple variants match, the first
/// one in the enum definition is used. No variant matching means the entire enum won't match.
///
/// The requirements for a format string on a variant are identical to those for a struct: Each
/// field of the variant, that is not annotated with `default`, must appear exactly once in the
/// format string. This means that the format string for `Variant1` must contain `<field>`,
/// the format string for `Variant2` must contain `{0}` or `{}` and the format string for
/// `Variant3` has no placeholders.
///
/// Any variants that don't have a format string (like `Variant4` in the example above) won't be
/// constructed by `sscanf`.
///
/// ## A note on Generics
/// Any lifetime parameters will be carried over. Any type `&'a str` will contain a borrow of the
/// input string, with an appropriate lifetime.
///
/// As for type generics: [`RegexRepresentation`](crate::RegexRepresentation) cannot be implemented
/// for generic types, since the contained associated `const` is only created once by Rust for all
/// generic instances, meaning that different regexes for different `T` are not possible. This
/// also means that deriving `FromScanf` for a struct that wants to match a generic field without
/// a `map` or `default` attribute will generally fail. The only possibilities are:
/// ```
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "...{field:/<regex>/}...")]
/// struct MyGenericStruct<T>
/// where
///     T: std::str::FromStr + 'static,
///     <T as std::str::FromStr>::Err: std::error::Error + 'static,
/// {
///     field: T,
/// }
///
/// let input = "...<regex>...";
/// let res = sscanf::sscanf!(input, "{MyGenericStruct<String>}").unwrap();
/// assert_eq!(res.field, String::from("<regex>"));
/// ```
/// There are two important things in this example:
/// 1. Since `RegexRepresentation` cannot be used, every occurrence of generic fields in the format
///    string have to have a regex (`{:/.../}) attached to them.
/// 2. The type bounds on `T` have to contain all of those exact bounds.
///
/// Any `T` has to be constructed by [`FromStr`](std::str::FromStr) from what is matched by the
/// specified regex, making this setup virtually useless for all but a few selected types. Since
/// the generic parameter has to be specified in the actual `sscanf` call, it is usually better
/// to just use a concrete type in the struct itself.
///
/// It is possible to have `T` directly require `FromScanf` like this: `T: for<'a> FromScanf<'a>`.
/// However, since `FromScanf` implementations usually rely on capture groups inside of their regex,
/// this would require also having the exact same capture groups in the format string, which is
/// currently not possible. Implementations that don't rely on capture groups are usually those
/// that were blanket-implemented based on their `FromStr` implementation.
pub use sscanf_macro::FromScanf;

#[doc(hidden)]
pub use sscanf_macro::FromScanf as FromSscanf;
