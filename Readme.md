# sscanf

A Rust crate with a sscanf (inverse of format!()) Macro based on Regex

[![Build](https://github.com/mich101mich/sscanf/actions/workflows/build.yml/badge.svg)](https://github.com/mich101mich/sscanf/actions/workflows/build.yml)
[![Tests](https://github.com/mich101mich/sscanf/actions/workflows/test.yml/badge.svg)](https://github.com/mich101mich/sscanf/actions/workflows/test.yml)

`sscanf` is originally a C-function that takes a String, a format String with placeholders and several
Variables (in the Rust version replaced with Types). It then parses the input String, writing
the values behind the placeholders into the Variables (Rust: returns a Tuple). `sscanf` can be
thought of as reversing a call to `format!()`:
```rust
// format: takes format string and values, returns String
let s = format!("Hello {}_{}!", "World", 5);
assert_eq!(s, "Hello World_5!");

// scanf: takes String, format string and types, returns Tuple
let parsed = sscanf::scanf!(s, "Hello {}_{}!", String, usize);
// parsed is Option<(String, usize)>
assert_eq!(parsed, Some((String::from("World"), 5)));
```
`scanf!()` takes a format String like `format!()`, but doesn't write
the values into the placeholders (`{}`), but extracts the values at those `{}` into the return Tuple.

If matching the format string failed, `None` is returned:
```rust
let s = "Text that doesn't match the format string";
let parsed = sscanf::scanf!(s, "Hello {}_{}!", String, usize);
assert_eq!(parsed, None); // No match possible
```

Note that the original C-function and this Crate are called sscanf, which is the technically
correct version in this context. `scanf` (with one `s`) is a similar C-function that reads a
console input instead of taking a String parameter. The macro itself is called `scanf!()` because
that is shorter, can be pronounced without sounding too weird and nobody uses the stdin version
anyway.

More examples of the capabilities of `scanf`:
```rust
use sscanf::scanf;

let input = "<x=3, y=-6, z=6>";
let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32);
assert_eq!(parsed, Some((3, -6, 6)));

let input = "4-5 t: ftttttrvts";
let parsed = scanf!(input, "{}-{} {}: {}", usize, usize, char, String);
assert_eq!(parsed, Some((4, 5, 't', String::from("ftttttrvts"))));

let input = "Move to N36E21";
let parsed = scanf!(input, "Move to {}{}{}{}", char, usize, char, usize);
assert_eq!(parsed, Some(('N', 36, 'E', 21)));

let input = "A Sentence. Another Sentence. Yet more Words with Spaces.";
let parsed = scanf!(input, "{}. {}. {}.", String, String, String);
assert!(parsed.is_some());
let (a, b, c) = parsed.unwrap();
assert_eq!(a, "A Sentence");
assert_eq!(b, "Another Sentence");
assert_eq!(c, "Yet more Words with Spaces");
```
The input in this case is a `&'static stc`, but in can be `String`, `&str`, `&String`, ... Basically
anything with `AsRef<str>` and without taking Ownership.

The parsing part of this macro has very few limitations, since it replaces the `{}` with a Regular
Expression ([`regex`](https://docs.rs/regex)) that corresponds to that type.
For example:
- `char` is just one Character (regex `"."`)
- `String` is any sequence of Characters (regex `".+"`)
- Numbers are any sequence of digits (regex `"\d+"`)

And so on. The actual implementation for numbers tries to take the size of the Type into
account and some other details, but that is the gist of the parsing.

This means that any sequence of replacements is possible as long as the Regex finds a
combination that works. In the `char, usize, char, usize` example above it manages to assign
the `N` and `E` to the `char`s because they cannot be matched by the `usize`s. If the input
were slightly different then it might have matched the `6` of the `36` or the `2` of the `21`
to the second `char`.

## Custom Types

`scanf` works with most of the primitive Types from `std` as well as `String` by default. The
full list can be seen here: [Implementations of `RegexRepresentation`](https://docs.rs/sscanf/^0/sscanf/trait.RegexRepresentation.html#foreign-impls).

More Types can easily be added, as long as they implement [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html) for the parsing
and [`RegexRepresentation`](https://docs.rs/sscanf/^0/sscanf/trait.RegexRepresentation.html) for `scanf` to obtain the Regex of the Type:
```rust
struct TimeStamp {
    year: usize, month: u8, day: u8,
    hour: u8, minute: u8,
}
impl sscanf::RegexRepresentation for TimeStamp {
    /// Matches "[year-month-day hour:minute]"
    const REGEX: &'static str = r"\[\d\d\d\d-\d\d-\d\d \d\d:\d\d\]";
}
impl std::str::FromStr for TimeStamp {
    // ...
}

let input = "[1518-10-08 23:51] Guard #751 begins shift";
let parsed = scanf!(input, "{} Guard #{} begins shift", TimeStamp, usize);
assert_eq!(parsed, Some((TimeStamp{
    year: 1518, month: 10, day: 8,
    hour: 23, minute: 51
}, 751)));
```
