# sscanf

A Rust crate with a sscanf-style Macro based on Regex

[![Build](https://github.com/mich101mich/sscanf/actions/workflows/build.yml/badge.svg)](https://github.com/mich101mich/sscanf/actions/workflows/build.yml)
[![Tests](https://github.com/mich101mich/sscanf/actions/workflows/test.yml/badge.svg)](https://github.com/mich101mich/sscanf/actions/workflows/test.yml)

TODO: Add Text here

```Rust
use sscanf::scanf;

let input = "4-5 t: ftttttrvts";
let parsed = scanf!(input, "{}-{} {}: {}", usize, usize, char, String);
assert_eq!(parsed, Some((4, 5, 't', String::from("ftttttrvts"))));

let input = "<x=3, y=-6, z=6>";
let parsed = scanf!(input, "<x={}, y={}, z={}>", i32, i32, i32);
assert_eq!(parsed, Some((3, -6, 6)));

let input = "Goto N36E21";
let parsed = scanf!(input, "Goto {}{}{}{}", char, usize, char, usize);
assert_eq!(parsed, Some(('N', 36, 'E', 21)));
```

## Custom Types
TODO: Add Text here
```Rust
struct TimeStamp {
    year: usize, month: u8, day: u8,
    hour: u8, minute: u8,
}
impl sscanf::RegexRepresentation for TimeStamp {
    const REGEX: &'static str = r"\d\d\d\d-\d\d-\d\d \d\d:\d\d";
}
impl std::str::FromStr for TimeStamp {
    // ...
}

let input = "[1518-10-08 23:51] Guard #751 begins shift";
let parsed = scanf!(input, "[{}] Guard #{} begins shift", TimeStamp, usize);
assert_eq!(parsed, Some((TimeStamp{
    year: 1518, month: 10, day: 8,
    hour: 23, minute: 51
}, 751)));
```

TODO: Add more here
