#![allow(dead_code)]

use sscanf::*;

#[test]
#[should_panic = r#"sscanf: Failed to compile regex: "regex parse error:\n    ^())$\n       ^\nerror: unopened group""#]
fn invalid_regex() {
    struct Test;
    impl FromScanf<'_> for Test {
        const REGEX: &'static str = ")";
        fn from_match(_: &str) -> Option<Self> {
            Some(Test)
        }
    }
    sscanf!("hi", "{Test}").unwrap();
}

#[test]
#[should_panic = "called `Option::unwrap()` on a `None` value"]
fn check_error_regex() {
    sscanf!("hi", "bob").unwrap();
}

#[test]
#[should_panic = "called `Option::unwrap()` on a `None` value"]
fn check_error_from_str_1() {
    sscanf!("5bobhibob", "{u32}bob{usize:/.*/}bob").unwrap();
}

#[test]
#[should_panic = "called `Option::unwrap()` on a `None` value"]
fn check_error_from_str_2() {
    struct Test(usize);
    impl FromScanf<'_> for Test {
        const REGEX: &'static str = ".*";
        fn from_match(s: &str) -> Option<Self> {
            s.parse().ok().map(Test)
        }
    }
    sscanf!("bobhibob", "bob{}bob", Test).unwrap();
}
