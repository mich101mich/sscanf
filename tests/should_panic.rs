#![allow(dead_code)]

use sscanf::{advanced::*, *};

macro_rules! assert_throws {
    ( $block:block, $message:expr $(,)? ) => {
        let error = std::panic::catch_unwind(move || $block).unwrap_err();
        if let Some(s) = error.downcast_ref::<&'static str>() {
            assert_eq!(*s, $message);
        } else if let Some(s) = error.downcast_ref::<String>() {
            assert_eq!(s, $message);
        } else {
            panic!("unexpected panic payload: {:?}", error);
        }
    };
    ( $expression:expr, $message:expr $(,)? ) => {
        assert_throws!(
            {
                $expression;
            },
            $message
        );
    };
}

#[test]
fn invalid_regex() {
    struct Test;
    impl FromScanfSimple<'_> for Test {
        const REGEX: &'static str = "asdf)hjkl";
        fn from_match(_: &str) -> Option<Self> {
            Some(Test)
        }
    }
    assert_throws!(
        sscanf!("hi", "{Test}").unwrap(),
        r#"sscanf: Invalid REGEX on FromScanfSimple of type should_panic::invalid_regex::Test: regex parse error:
    asdf)hjkl
        ^
error: unopened group"#
    );
}

#[test]
fn check_error_regex() {
    assert_throws!(
        sscanf!("hi", "bob").unwrap(),
        "called `Option::unwrap()` on a `None` value"
    );
}

#[test]
fn check_error_from_str_1() {
    assert_throws!(
        sscanf!("5bobhibob", "{u32}bob{usize:/.*/}bob").unwrap(),
        "called `Option::unwrap()` on a `None` value"
    );
}

#[test]
fn check_error_from_str_2() {
    struct Test(usize);
    impl FromScanfSimple<'_> for Test {
        const REGEX: &'static str = ".*";
        fn from_match(s: &str) -> Option<Self> {
            s.parse().ok().map(Test)
        }
    }
    assert_throws!(
        sscanf!("bobhibob", "bob{}bob", Test).unwrap(),
        "called `Option::unwrap()` on a `None` value"
    );
}

#[test]
fn nesting() {
    mod my_mod {
        // FIXME: There are currently (Rust stable 1.90.0, nightly 1.93.0-nightly (34f954f9b 2025-10-25)) differences
        //        between the panic message on stable and nightly rust if there are lifetimes involved.
        // pub struct MyType<'bob, LeGenerics>(pub &'bob LeGenerics);
        pub struct MyType<LeGenerics>(pub LeGenerics);
    }
    static R: Vec<usize> = vec![];
    use my_mod::MyType;
    // impl FromScanf<'_> for MyType<'static, std::vec::Vec<usize>> {
    impl FromScanf<'_> for MyType<std::vec::Vec<usize>> {
        fn get_matcher(_: &FormatOptions) -> Matcher {
            Matcher::Seq(vec![
                MatchPart::literal("a"),
                MatchPart::Matcher(Matcher::from_regex("b").unwrap().optional()),
                MatchPart::literal("c"),
            ])
        }
        fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
            let _ = matches.as_seq().at(1).as_opt()?.as_raw().get(0).unwrap();
            // Some(MyType(&R))
            Some(MyType(vec![]))
        }
    }
    assert_throws!(
        sscanf_unescaped!("abc", "{MyType<_>}").unwrap(),
        r#"sscanf: index 0 is out of bounds of 0 captures in RawMatch::get.
Context: sscanf -> as_seq() -> parse 0 as should_panic::nesting::my_mod::MyType<alloc::vec::Vec<usize>> -> as_seq() -> at(1) -> as_opt() -> as_raw() -> get(0)"#
    );
}
