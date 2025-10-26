#![allow(dead_code)]

use sscanf::{advanced::*, *};

#[test]
#[should_panic = r#"sscanf: Invalid REGEX on FromScanfSimple of type should_panic::invalid_regex::Test: regex parse error:
    asdf)hjkl
        ^
error: unopened group"#]
fn invalid_regex() {
    struct Test;
    impl FromScanfSimple<'_> for Test {
        const REGEX: &'static str = "asdf)hjkl";
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
    impl FromScanfSimple<'_> for Test {
        const REGEX: &'static str = ".*";
        fn from_match(s: &str) -> Option<Self> {
            s.parse().ok().map(Test)
        }
    }
    sscanf!("bobhibob", "bob{}bob", Test).unwrap();
}

#[test]
#[should_panic = "sscanf: inner match at index 2 is None. Are there any unescaped `?` or `|` in a regex?
Context: sscanf -> parse group 0 as should_panic::nesting::my_mod::MyType<alloc::vec::Vec<usize>> -> assert group 0 -> get group 1 -> parse group 2 as should_panic::nesting::my_mod::MyType<alloc::vec::Vec<usize>>"]
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
            Matcher::from_regex("a(b()(c()()(d)?))").unwrap()
        }
        fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
            matches
                .at(0)
                .get(1)
                .unwrap()
                .parse_at::<MyType<_>>(2, &Default::default())
                .unwrap();
            // Some(MyType(&R))
            Some(MyType(vec![]))
        }
    }
    sscanf_unescaped!("abc", "{MyType<_>}").unwrap();
}
