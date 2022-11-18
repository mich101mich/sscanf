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
/// A [`Result`](std::result::Result) with a tuple of the parsed types or a [`sscanf::Error`](crate::Error).
/// Note that an error usually indicates that the input didn't match the format string, making the
/// returned [`Result`](std::result::Result) functionally equivalent to an [`Option`](std::option::Option),
/// and most applications should treat it that way. An error is only useful when debugging
/// custom implementations of [`FromStr`](std::str::FromStr) or [`FromScanf`](crate::FromScanf).
/// See [`sscanf::Error`](crate::Error) for more information.
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
/// #[sscanf(format = "<format>")]
/// struct MyStruct {
///     <field>: <type>, // requires <type>: FromScanf
///
///     <field_2>: <type_2>, // requires <type_2>: FromScanf
///
///     // ...
///
///     // possible attributes on fields:
///
///     #[sscanf(default)]
///     <field_with_default>: <type>, // requires <type>: Default
///
///     #[sscanf(default = <expression>)] // accepts any expression that returns <type>
///     <field_with_custom_default>: <type>,
///
///     #[sscanf(map = |input: <matched_type>| { <conversion from <matched_type> to <actual_type>> })]
///     <field_with_conversion>: <actual_type>, // requires <matched_type>: FromScanf
/// }
///
/// // tuple structs have the same capabilities, just without field names:
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "<format>")]
/// struct MyTupleStruct(<type>, #[sscanf(default)] <type>, ...);
/// ```
///
/// **<format>**: The format string to parse the struct from. Similar to the format string for
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
/// Mapping allows matching against a different type than the field type. The `map` attribute takes
/// a closure that takes the matched type as input and returns the field type. The type parameter
/// of the closure has to be explicitly specified, since it is needed to generate the matching
/// code.
///
/// The types of the used fields of their matching types have to implement [`FromScanf`](crate::FromScanf)
/// and either [`RegexRepresentation`](crate::RegexRepresentation) or have a `{<field>:/<regex>/}`
/// placeholder.
pub use sscanf_macro::FromScanf;
