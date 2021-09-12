# sscanf

A Rust crate with a sscanf (inverse of format!()) Macro based on Regex

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

let input = "Move to N36E21";
let parsed = scanf!(input, "Move to {}{}{}{}", char, usize, char, usize);
assert_eq!(parsed, Some(('N', 36, 'E', 21)));

let input = "Escape literal { } as {{ and }}";
let parsed = scanf!(input, "Escape literal {{ }} as {{{{ and {}", String);
assert_eq!(parsed, Some(String::from("}}")));

let input = "A Sentence with Spaces. Number formats: 0xab01 0o127 0b101010.";
let parsed = scanf!(input, "{}. Number formats: {x} {o} {b}.", String, usize, i32, u8);
let (a, b, c, d) = parsed.unwrap();
assert_eq!(a, "A Sentence with Spaces");
assert_eq!(b, 0xab01);
assert_eq!(c, 0o127);
assert_eq!(d, 0b101010);
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

## Format Options
All Options are inside `'{'` `'}'`. Literal `'{'` or `'}'` inside of a Format Option are escaped
as `'\{'` instead of `'{{'` to avoid ambiguity.

Procedural macro don't have any reliable type information, so the Type must be the exact required
Type without any path or alias (`chrono` imports happen automatically)

**Radix Options:**

Only work on primitive number types (u8, i8, u16, ...).
- `x`: hexadecimal Number (Digits 0-9 and A-F, optional Prefix `0x`)
- `o`: octal Number (Digits 0-7, optional Prefix `0o`)
- `b`: binary Number (Digits 0-1, optional Prefix `0b`)
- `r2` - `r36`: any radix Number

**[`chrono`](https://docs.rs/chrono/^0.4/chrono/) integration (Requires `chrono` feature):**

The types [`DateTime`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html),
[`NaiveDate`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveDate.html),
[`NaiveTime`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveTime.html),
[`NaiveDateTime`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveDateTime.html),
[`Utc`](https://docs.rs/chrono/^0.4/chrono/offset/struct.Utc.html) and
[`Local`](https://docs.rs/chrono/^0.4/chrono/offset/struct.Local.html) can be used and accept
a [Date/Time format string](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html)
inside of the `{` `}`, that will then be used for both the Regex generation and parsing of the
type.

Using [`DateTime`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html) returns a
`DateTime<FixedOffset>` and requires the rules and limits that [`DateTime::parse_from_str`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html#method.parse_from_str)
has.

```rust
use chrono::prelude::*;

let input = "10:37:02";
let parsed = scanf!(input, "{%H:%M:%S}", NaiveTime);
assert_eq!(parsed, Some(NaiveTime::from_hms(10, 37, 2)));

let input = "Today is the 23. of May, 2020 at 09:05 pm and 7 seconds.";
let parsed = scanf!(input, "Today is the {%d. of %B, %Y at %I:%M %P and %-S} seconds.", Utc);
assert_eq!(parsed, Some(Utc.ymd(2020, 5, 23).and_hms(21, 5, 7)));
```

Note: The `chrono` feature needs to be active for this to work, because `chrono` is an optional dependency

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
