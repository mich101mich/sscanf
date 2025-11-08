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

#[test]
#[should_panic = "sscanf: inner match at index 0 is None. Are there any unescaped `?` or `|` in a regex?
Context: sscanf -> parse group 0 as u8"]
fn optional_capture_group() {
    sscanf_unescaped!("", "{u8}?").unwrap();
}

#[test]
#[should_panic = "sscanf: inner match at index 0 is None. Are there any unescaped `?` or `|` in a regex?
Context: sscanf -> parse group 0 as u8"]
fn alternative_capture_group() {
    sscanf_unescaped!("abc", "{u8}|{str}").unwrap();
}

#[test]
#[should_panic = "sscanf: inner match at index 2 is None. Are there any unescaped `?` or `|` in a regex?
Context: sscanf -> parse group 0 as should_panic::nesting::my_mod::MyType<alloc::vec::Vec<usize>> -> assert group 0 -> get group 1 -> parse group 2 as should_panic::nesting::my_mod::MyType<alloc::vec::Vec<usize>>"]
fn nesting() {
    mod my_mod {
        pub struct MyType<'bob, LeGenerics>(pub &'bob LeGenerics);
    }
    static R: Vec<usize> = vec![];
    use my_mod::MyType;
    impl FromScanf<'_> for MyType<'static, std::vec::Vec<usize>> {
        const REGEX: &'static str = "a(b()(c()()(d)?))";
        fn from_match(_: &str) -> Option<Self> {
            None
        }
        fn from_match_tree(matches: MatchTree<'_, '_>) -> Option<Self> {
            matches
                .at(0)
                .get(1)
                .unwrap()
                .parse_at::<MyType<_>>(2)
                .unwrap();
            Some(MyType(&R))
        }
    }
    sscanf_unescaped!("abc", "{MyType<_>}").unwrap();
}

#[test]
#[should_panic = "sscanf: inner match at index 0 is None. Are there any unescaped `?` or `|` in a regex?
Context: sscanf -> parse group 0 as should_panic::nesting2::MyUnnamedType -> parse field .1 from group 1 as should_panic::nesting2::MyNamedType -> parse field .x from group 0 as char"]
fn nesting2() {
    #[derive(FromScanf)]
    #[sscanf(r"x={x}?")]
    struct MyNamedType {
        x: char,
    }

    #[derive(FromScanf)]
    #[sscanf("{}: {}")]
    struct MyUnnamedType(usize, MyNamedType);

    sscanf_unescaped!("3: x=", "{MyUnnamedType}").unwrap();
}
