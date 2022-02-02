# sscanf

A Rust crate with a sscanf (inverse of format!()) Macro based on Regex

[![Tests](https://github.com/mich101mich/sscanf/actions/workflows/test.yml/badge.svg)](https://github.com/mich101mich/sscanf/actions/workflows/test.yml)

`sscanf` is originally a C-function that takes a String, a format String with placeholders and several
Variables (in the Rust version replaced with Types). It then parses the input String, writing
the values behind the placeholders into the Variables (Rust: returns a Tuple). `sscanf` can be
thought of as reversing a call to `format!()`:
```rust
// format: takes format string and values, returns String
let s = format!("Hello {}{}!", "World", 5);
assert_eq!(s, "Hello World5!");

// scanf: takes String, format string and types, returns Tuple
let parsed = scanf!(s, "Hello {}{}!", String, usize);

// parsed is Result<(String, usize), sscanf::Error>
assert_eq!(parsed, Ok((String::from("World"), 5)));
```
`scanf!()` takes a format String like `format!()`, but doesn't write
the values into the placeholders (`{}`), but extracts the values at those `{}` into the return Tuple.

If matching the format string failed, an Error is returned:
```rust
let s = "Text that doesn't match the format string";
let parsed = scanf!(s, "Hello {}{}!", String, usize);
assert!(matches!(parsed, sscanf::Error::RegexMatchFailed{..}));
```

Note that the original C-function and this Crate are called sscanf, which is the technically
correct version in this context. `scanf` (with one `s`) is a similar C-function that reads a
console input instead of taking a String parameter. The macro itself is called `scanf!()` because
that is shorter, can be pronounced without sounding too weird and nobody uses the stdin version
anyway.

More examples of the capabilities of `scanf`:
```rust
let input = "<x=3, y=-6, z=6>";
let parsed = scanf!(input, "<x={i32}, y={i32}, z={i32}>"); // types can be written inside placeholders
assert_eq!(parsed, Ok((3, -6, 6)));

let input = "Move to N36E21";
let parsed = scanf!(input, "Move to {char}{usize}{char}{usize}");
assert_eq!(parsed, Ok(('N', 36, 'E', 21)));

let input = "Escape literal { } as {{ and }}";
let parsed = scanf!(input, "Escape literal {{ }} as {{{{ and }}}}");
assert_eq!(parsed, Ok(()));

let input = "A Sentence with Spaces. Another Sentence.";
let (a, b) = scanf!(input, "{String}. {String}.").unwrap();
assert_eq!(a, "A Sentence with Spaces");
assert_eq!(b, "Another Sentence");

let input = "Formats:  0xab01  0o127  101010  1Z";
let parsed = scanf!(input, "Formats:  {usize:x}  {i32:o}  {u8:b}  {u32:r36}");
let (a, b, c, d) = parsed.unwrap();
assert_eq!(a, 0xab01);     // Hexadecimal
assert_eq!(b, 0o127);      // Octal
assert_eq!(c, 0b101010);   // Binary

assert_eq!(d, 71);         // any radix (r36 = Radix 36)
assert_eq!(d, u32::from_str_radix("1Z", 36).unwrap());

let input = "color: #D4AF37";
// Number types take their size into account, and hexadecimal u8 can have at most 2 digits.
// => the only possible match is 2 digits each.
let (r, g, b) = scanf!(input, "color: #{u8:x}{u8:x}{u8:x}").unwrap();
assert_eq!((r, g, b), (0xD4, 0xAF, 0x37));
```
The input here is a `&'static str`, but in can be `String`, `&str`, `&String`, ...
Basically anything with [`AsRef<str>`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
and without taking Ownership.

The parsing part of this macro has very few limitations, since it replaces the `{}` with a Regular
Expression ([`regex`](https://docs.rs/regex)) that corresponds to that type.
For example:
- `char` is just one Character (regex `"."`)
- `String` is any sequence of Characters (regex `".+?"`)
- Numbers are any sequence of digits (regex `"[-+]?\d+"`)

And so on. The actual implementation for numbers tries to take the size of the Type into
account and some other details, but that is the gist of the parsing.

This means that any sequence of replacements is possible as long as the Regex finds a
combination that works. In the `char, usize, char, usize` example above it manages to assign
the `N` and `E` to the `char`s because they cannot be matched by the `usize`s.

## Format Options
All Options are inside `'{'` `'}'` and after a `:`. Literal `'{'` or `'}'` inside of a Format
Option are escaped as `'\{'` instead of `'{{'` to avoid ambiguity.

Procedural macro don't have any reliable type info and can only compare types by name. This means
that the number options below only work with a literal type like "`i32`", **NO** Paths (~~`std::i32`~~)
or Wrappers (~~`struct Wrapper(i32);`~~) or Aliases (~~`type Alias = i32;`~~). **ONLY** `i32`,
`usize`, `u16`, ...

| config                      | description                | possible types |
| --------------------------- | -------------------------- | -------------- |
| `{:/` _\<regex>_ `/}`       | custom regex               | any            |
| `{:x}`                      | hexadecimal numbers        | numbers        |
| `{:o}`                      | octal numbers              | numbers        |
| `{:b}`                      | binary numbers             | numbers        |
| `{:r2}` - `{:r36}`          | radix 2 - radix 32 numbers | numbers        |
| `{:` _\<chrono format>_ `}` | chrono format              | chrono types   |

**Custom Regex:**

- `{:/.../}`: Match according to the [`Regex`](https://docs.rs/regex) between the `/` `/`

For example:
```rust
let input = "random Text";
let (a, b) = scanf!(input, "{String:/[^m]+/}{String}").unwrap();

// regex  [^m]+  matches anything that isn't an 'm'
// => stops at the 'm' in 'random'
assert_eq!(a, "rando");
assert_eq!(b, "m Text");
```

As mentioned above, `'{'` `'}'` have to be escaped with a `'\'`. This means that:
- `"{"` or `"}"` would give a compiler error
- `"\{"` or `"\}"` lead to a `"{"` or `"}"` inside of the regex
  - curly brackets have a special meaning in regex as counted repetition
- `"\\{"` or `"\\}"` would give a compiler error
  - first `'\'` escapes the second one, leaving just the brackets
- `"\\\{"` or `"\\\}"` lead to a `"\{"` or `"\}"` inside of the regex
  - the first `'\'` escapes the second one, leading to a literal `'\'` in the regex. the third
    escapes the curly bracket as in the second case
  - needed in order to have the regex match an actual curly bracket

Note that this is only the case if you are using raw strings for formats, regular strings require
escaping `'\'`, so this would double the number of `'\\'`.

Works with non-`String` types too:
```rust
let input = "Match 4 digits of 123456";
let parsed = scanf!(input, r"Match 4 digits of {usize:/\d\{4\}/}{usize}");
                           // raw string (r"") to write \d instead of \\d

// regex  \d{4}  matches 4 digits
assert_eq!(parsed, Ok((1234, 56)));
```
Note that changing the regex of a non-`String` type might cause that type's [`FromStr`](https://doc.rust-lang.org/std/str/trait.FromStr.html)
to fail

**Number Options:**

Only work on primitive number types (`u8`, ..., `u128`, `i8`, ..., `i128`, `usize`, `isize`):
- `x`: hexadecimal Number (Digits 0-9 and A-F or a-f, optional Prefix `0x`)
- `o`: octal Number (Digits 0-7, optional Prefix `0o`)
- `b`: binary Number (Digits 0-1, optional Prefix `0b`)
- `r2` - `r36`: any radix Number (no prefix)

**[`chrono`](https://docs.rs/chrono/^0.4/chrono/) integration (Requires `chrono` feature):**

The types [`DateTime`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html),
[`NaiveDate`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveDate.html),
[`NaiveTime`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveTime.html),
[`NaiveDateTime`](https://docs.rs/chrono/^0.4/chrono/naive/struct.NaiveDateTime.html),
[`Utc`](https://docs.rs/chrono/^0.4/chrono/offset/struct.Utc.html) and
[`Local`](https://docs.rs/chrono/^0.4/chrono/offset/struct.Local.html) can be used and accept
a [Date/Time format string](https://docs.rs/chrono/0.4.19/chrono/format/strftime/index.html)
inside of the `{` `}`. This will then be used for both the Regex generation and parsing of the
type.

Using [`DateTime`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html) returns a
`DateTime<FixedOffset>` and requires the rules and limits that [`DateTime::parse_from_str`](https://docs.rs/chrono/^0.4/chrono/struct.DateTime.html#method.parse_from_str)
has.

```rust
use chrono::prelude::*;

let input = "10:37:02";
let parsed = scanf!(input, "{NaiveTime:%H:%M:%S}");
assert_eq!(parsed, Ok(NaiveTime::from_hms(10, 37, 2)));

let expected = Utc.ymd(2020, 5, 23).and_hms(21, 5, 7);

// DateTime<*> directly implements FromStr and doesn't need a config
let input = "2020-05-23T21:05:07Z";
let parsed = scanf!(input, "{DateTime<Utc>}");
assert_eq!(parsed, Ok(expected));

let input = "Today is the 23. of May, 2020 at 09:05 pm and 7 seconds.";
let parsed = scanf!(input, "Today is the {Utc:%d. of %B, %Y at %I:%M %P and %-S} seconds.");
assert_eq!(parsed, Ok(expected));
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
let parsed = scanf!(input, "{TimeStamp} Guard #{usize} begins shift");
assert_eq!(parsed, Ok((TimeStamp{
    year: 1518, month: 10, day: 8,
    hour: 23, minute: 51
}, 751)));
```

Implementing `RegexRepresentation` isn't _strictly_ necessary if you **always** supply a custom
Regex when using the type by using the `{:/.../}` format option, but this tends to make your code
less readable.
