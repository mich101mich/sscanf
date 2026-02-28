//! Macro re-exports to keep documentation separate from the crate root.

#[expect(unused_imports, reason = "for doc links")]
use crate::Parser;

/// Parses a string using a format string, similar to C's `sscanf`.
///
/// ## Signature
/// ```ignore
/// sscanf!(input: impl Deref<Target=str>, format: <literal>, Type...) -> Option<(Type...)>
/// ```
///
/// ## Parameters
/// * `input`: The string to parse. Can be anything that auto-derefs to `str` (e.g. `&str`, `String`, `Cow<str>`, etc.).
///   See the examples below. Note that `sscanf` does not take ownership of the input.\
///   More formally, `sscanf` adds a `&` before the input and then passes it to [`Parser::parse`](crate::Parser::parse),
///   so the input must be something that rust can coerce to `&str`.
/// * `format`: A literal string. No `const` or `static` allowed, just like with [`format!()`](std::format).
/// * `Type...`: Any types that are not written into the format string. See [Custom Types](index.html#custom-types)
///   for details.
///
/// ## Return Value
/// Returns `Some(tuple)` containing the parsed types, or `None` if matching or parsing fails.
///
/// ## Details
/// The format string must be a string literal, because it is parsed by the procedural macro at compile time to ensure
/// all types and placeholders match. This cannot be done if the format is stored in a variable or even a `const &str`
/// elsewhere.
///
/// Placeholders in the format string are `{}`. Any `{` or `}` that should be literal must be escaped as `{{` and `}}`.
/// For every placeholder there must be a type name inside the `{}`, or exactly one type in the parameters after the
/// format string. Types can be referenced by indices like `{0}`, similar to [`format!()`](std::format).
///
/// Any additional formatting options are placed after a `:`. For a list of options, see the
/// [crate root documentation](index.html#format-options).
///
/// ## Examples
/// Examples of accepted input types:
/// ```
/// # use sscanf::sscanf; use std::{borrow::Cow, rc::Rc, boxed::Box};
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
/// let input: Box<str> = String::from("5").into_boxed_str();
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input: Cow<str> = Cow::Borrowed("5");
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// let input: Rc<str> = Rc::from(String::from("5"));
/// assert_eq!(sscanf!(input, "{usize}").unwrap(), 5);
///
/// // and many more
/// ```
///
/// ```compile_fail
/// // temporary value: does not work
/// # use sscanf::sscanf;
/// sscanf!(String::from("5"), "{usize}");
/// ```
///
/// More examples are available in the crate root documentation.
pub use sscanf_macro::sscanf;

/// Same as [`sscanf`], but allows using regex in the format string.
///
/// Signature is the same as [`sscanf`].
///
/// Parameters are the same as [`sscanf`], but any non-placeholder parts of the format string are treated as regex.
///
/// Note that the `{{` and `}}` escaping for literal `{` and `}` is still required. So if you want to have a
/// counted repetition like `[0-9]{4}` as part of the regex, you have to write it as `[0-9]{{4}}`.
///
/// The pattern is automatically anchored: `^` at the start and `$` at the end.
///
/// ## Examples
/// ```
/// use sscanf::sscanf_with_regex;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = sscanf_with_regex!(input, "{f32}.*?{usize}"); // .*? matches anything
/// assert_eq!(output.unwrap(), (5.0, 3));
/// ```
///
/// The basic [`sscanf`] escapes `.`, `*`, and `?` and matches literal characters:
/// ```
/// use sscanf::sscanf;
/// let input = "5.0SOME_RANDOM_TEXT3";
/// let output = sscanf!(input, "{f32}.*{usize}");
/// assert!(output.is_none()); // does not match
///
/// let input2 = "5.0[a-z]3";
/// let output2 = sscanf!(input2, "{f32}[a-z]{usize}"); // regular sscanf is unaffected by special characters
/// assert_eq!(output2.unwrap(), (5.0, 3));
/// ```
pub use sscanf_macro::sscanf_with_regex;

/// Converts a format string and types into a [`Parser`] that can be reused to parse multiple inputs.
///
/// ## Signature
/// ```ignore
/// sscanf_parser!(format: <literal>, Type...) -> Parser<(Type...)>
/// ```
///
/// ## Parameters
/// * `format`: A literal string. No `const` or `static` allowed, just like with [`format!()`](std::format).
/// * `Type...`: Any types that are not written into the format string. See [Custom Types](index.html#custom-types)
///   for details.
///
/// ## Return Value
/// Returns a [`Parser`] over the parsed tuple. Use the [`parse`](crate::Parser::parse) method to parse inputs.
///
/// ## Details
/// Code like
/// ```ignore
/// sscanf!(input, "<format>", Type...)
/// ```
/// is identical to
/// ```ignore
/// sscanf_parser!("<format>", Type...).parse(input)
/// ```
///
/// Parsers can be reused across inputs. Creating a parser is semi-costly, so reusing it avoids overhead in loops with
/// the same format and types.
///
/// For types that borrow from the input string, lifetimes restrict how long a parser may live. See [`Parser`]
/// documentation for details.
///
/// ## Examples
/// ```
/// use sscanf::{sscanf_parser, Parser};
/// let mut parser: Parser<(&str, i32)> = sscanf_parser!("{&str} {i32}");
///
/// let multiline_input = "Hello 1\nWorld 2\nRust 3";
/// let parsed = multiline_input
///     .lines()
///     .filter_map(|line| parser.parse(line))
///     .collect::<Vec<_>>();
///
/// assert_eq!(parsed, vec![("Hello", 1), ("World", 2), ("Rust", 3)]);
/// ```
pub use sscanf_macro::sscanf_parser;

/// Same as [`sscanf_parser`], but allows using regex in the format string.
///
/// Signature and parameters are the same as [`sscanf_parser`]; format string handling matches
/// [`sscanf_with_regex`].
///
/// ## Examples
/// ```
/// use sscanf::sscanf_parser_with_regex;
/// let mut parser = sscanf_parser_with_regex!(r"{&str}\s+{i32}");
///
/// let aligned_table = r#"Name   Integer
/// Hello        1
/// World        2
/// of     3456789
/// Rust        10"#;
///
/// let parsed = aligned_table
///     .lines()
///     .skip(1) // skip header line
///     .map(|line| parser.parse(line).unwrap())
///     .collect::<Vec<_>>();
///
/// assert_eq!(parsed, vec![("Hello", 1), ("World", 2), ("of", 3456789), ("Rust", 10)]);
/// ```
pub use sscanf_macro::sscanf_parser_with_regex;

/// Derive macro for [`FromScanf`](crate::FromScanf).
///
/// ## For structs
/// ```ignore
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "<format>")] // format string; must contain placeholders for all
/// struct MyStruct {              // non-default fields: {<field>}, {<field_2>}, {<field_with_conversion>}
///
///     <field>: <type>, // requires <type> to implement FromScanf
///
///     <field_2>: <type_2>, // requires `<type_2>: FromScanf`
///
///     // ...
///
///     // possible attributes on fields:
///
///     #[sscanf(default)]
///     <field_with_default>: <type>, // requires `<type>: Default`, but doesn't need FromScanf
///
///     #[sscanf(default = <expression>)] // accepts any expression that returns <type>
///     <field_with_custom_default>: <type>, // no traits required.
///
///     #[sscanf(map = |input: <matched_type>| { <conversion from <matched_type> to <actual_type>> })]
///     <field_with_conversion>: <actual_type>, // requires `<matched_type>: FromScanf`
/// }
///
/// // tuple structs have the same capabilities, just without field names:
/// #[derive(sscanf::FromScanf)]
/// #[sscanf(format = "<format>")] // format string references fields by index: {0}, ...
/// struct MyTupleStruct(<type>, #[sscanf(default)] <type>, ...);
/// ```
///
/// ### Attributes
///
/// #### On the struct
///
/// - `format`: The format string to parse the struct from. Similar to the format string for [`sscanf`], but with field
///   names or indices instead of types. For a struct with fields `a`, `b`, and `c`, a format could be
///   `"{a} {b:/.*?/} {c}"`. All non-`default` fields must appear exactly once. Names or indices can be omitted when
///   fields are in the same order as `{}` placeholders, so the above can also be `"{} {:/.*?/} {}"`.
/// - `format_regex`: Same as `format`, but allows using regex in the format string. See [`sscanf_with_regex`] for
///   details.
/// - `transparent`: If the struct has exactly one field, the struct will be constructed from the field directly. This
///   is useful for newtype structs, where the struct is just a wrapper around another type. The field has to implement
///   [`FromScanf`](crate::FromScanf).
///
/// Only one of the above attributes can be used on a struct at a time. The `format = ` part can be omitted,
/// so `#[sscanf("<format>")]` is also valid.
///
/// #### On the fields
/// - `default` or `default = <expression>`: Sets the field from a default value rather than the input string.
///   `#[sscanf(default)]` uses [`Default::default()`](Default). If the type doesn't implement [`Default`], provide an
///   expression to compute the value. The expression can be any code, including function calls or `{ <code> }` blocks,
///   as long as it produces the field type.
/// - `map = |<param>: <type>| <conversion>`: Matches a different type and maps it to the field type. The closure's
///   parameter type must be specified, since it is needed for matching.
/// - `filter_map = |<param>: <type>| <conversion>`: Like `map`, but returns an [`Option`]. If it returns [`None`],
///   parsing fails.
/// - `from = <type>`: Matches a different type that implements [`FromScanf`] and converts it using [`From`].
/// - `try_from = <type>`: Like `from`, but the conversion can fail. If it does, parsing fails.
///
/// ## For enums
/// ```ignore
/// #[derive(sscanf::FromScanf)]
/// enum MyEnum {
///     #[sscanf(format = "<format>")] // has to contain `{<field>}` and any other fields
///     Variant1 {
///         <field>: <type>, // requires `<type>: FromScanf`
///
///         #[sscanf(default)]
///         <field_with_default>: <type2>, // requires `<type2>: Default`
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
/// An enum takes multiple format strings - one per variant. The value returned from `sscanf` is constructed from the
/// variant that matches the input. If multiple variants match, the first one in the enum definition is used. If no
/// variant matches, parsing fails.
///
/// ### Attributes
///
/// #### On the enum
/// - `autogen = "<case>"` or `autogenerate = "<case>"`: Automatically create format strings for all variants
///   based on the variant names. This only works for variants without fields. You can override the format by adding
///   a format attribute on the variant. The `case` parameter can be one of:
///   - `"CaseSensitive"`: The variant name is used as-is. Default if no `case` parameter is specified.
///   - `"CaseInsensitive"`: Same as `"CaseSensitive"`, but case is ignored.
///   - `"lower case"`: Lower case with spaces between words.
///   - `"UPPER CASE"`: Upper case with spaces between words.
///   - `"lowercase"`: Lower case, but without spaces.
///   - `"UPPERCASE"`: Upper case, but without spaces.
///   - `"PascalCase"`
///   - `"camelCase"`
///   - `"snake_case"`
///   - `"SCREAMING_SNAKE_CASE"`
///   - `"kebab-case"`
///   - `"SCREAMING-KEBAB-CASE"`
///
/// #### On the variants
///
/// Same as for structs. If neither a format string nor `transparent` is specified, the variant won't be constructed
/// by `sscanf`. With `autogen`, a format string is generated automatically; to prevent this, add `skip` to the variant.
/// `skip` has no effect without `autogen`.
///
pub use sscanf_macro::FromScanf;
