#![allow(dead_code)]

use sscanf::*;
use std::str::FromStr;

#[test]
#[should_panic(expected = "sscanf: Cannot generate Regex")]
fn invalid_regex_representation() {
    struct Test;
    impl FromStr for Test {
        type Err = std::convert::Infallible;
        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Test)
        }
    }
    impl RegexRepresentation for Test {
        const REGEX: &'static str = ")";
    }
    sscanf!("hi", "{Test}").unwrap();
}

#[test]
#[should_panic(expected = "MatchFailed")]
fn check_error_regex() {
    sscanf!("hi", "bob").unwrap();
}

#[test]
#[should_panic(
    expected = "FromStrFailedError { type_name: \"usize\", error: ParseIntError { kind: InvalidDigit } }"
)]
fn check_error_from_str_1() {
    sscanf!("5bobhibob", "{u32}bob{usize:/.*/}bob").unwrap();
}

#[test]
#[should_panic(
    expected = "FromStrFailedError { type_name: \"should_panic::check_error_from_str_2::Test\", error: ParseIntError { kind: InvalidDigit } }"
)]
fn check_error_from_str_2() {
    struct Test(usize);
    impl FromStr for Test {
        type Err = <usize as FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse().map(Test)
        }
    }
    impl RegexRepresentation for Test {
        const REGEX: &'static str = ".*";
    }
    sscanf!("bobhibob", "bob{}bob", Test).unwrap();
}

#[test]
#[should_panic(expected = r#"sscanf: Regex has 3 capture groups, but 2 were expected.
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid forming a capture group like this:
    "  (  )  "  =>  "  (?:  )  "
"#)]
fn custom_regex_with_capture_group() {
    struct Test(usize);
    impl FromStr for Test {
        type Err = <usize as FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse().map(Test)
        }
    }
    impl RegexRepresentation for Test {
        const REGEX: &'static str = "(.*)";
    }
    sscanf!("5", "{Test}").unwrap();
}
