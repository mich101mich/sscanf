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
        pub struct MyType<'bob, LeGenerics>(pub &'bob LeGenerics);
    }
    static R: Vec<usize> = vec![];
    use my_mod::MyType;
    impl FromScanf<'_> for MyType<'static, std::vec::Vec<usize>> {
        fn get_matcher(_: &FormatOptions) -> Matcher {
            Matcher::Seq(vec![
                MatchPart::literal("a"),
                MatchPart::Matcher(Matcher::from_regex("b").unwrap().optional()),
                MatchPart::literal("c"),
            ])
        }
        fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
            let _ = matches.as_seq().at(1).as_opt()?.as_alt().get();
            Some(MyType(&R))
        }
    }
    assert_throws!(
        sscanf_unescaped!("abc", "{MyType<_>}").unwrap(),
        if rustc_version::version().unwrap() < rustc_version::Version::new(1, 90, 0) {
            r#"sscanf: MatchTree::as_alt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::nesting::my_mod::MyType<alloc::vec::Vec<usize>> -> as_seq() -> at(1) -> as_opt()"#
        } else {
            r#"sscanf: MatchTree::as_alt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::nesting::my_mod::MyType<'_, alloc::vec::Vec<usize>> -> as_seq() -> at(1) -> as_opt()"#
        }
    );
}

mod context {
    use super::*;

    struct FailingStruct;
    impl FromScanf<'_> for FailingStruct {
        fn get_matcher(_: &FormatOptions) -> Matcher {
            Matcher::from_regex(".*").unwrap()
        }
        fn from_match_tree(matches: MatchTree<'_, '_>, _: &FormatOptions) -> Option<Self> {
            matches.as_opt();
            None
        }
    }

    struct SeqAtStruct(FailingStruct);
    impl FromScanf<'_> for SeqAtStruct {
        fn get_matcher(f: &FormatOptions) -> Matcher {
            Matcher::Seq(vec![
                MatchPart::literal("a"),
                FailingStruct::get_matcher(f).into(),
                MatchPart::literal("c: "),
                usize::get_matcher(f).into(),
            ])
        }
        fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
            let matches = matches.as_seq();
            let index = matches.parse_at(3, f)?;
            matches.at(index).parse::<FailingStruct>(f).map(SeqAtStruct)
        }
    }

    #[test]
    fn root() {
        assert_throws!(
            sscanf!("hi", "{FailingStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::FailingStruct"#
        );
    }

    #[test]
    fn parse() {
        struct ParseStruct(FailingStruct);
        impl FromScanf<'_> for ParseStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                FailingStruct::get_matcher(f)
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches.parse::<FailingStruct>(f).map(ParseStruct)
            }
        }

        assert_throws!(
            sscanf!("abc", "{ParseStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::parse::ParseStruct -> parse as should_panic::context::FailingStruct"#
        );
    }

    #[test]
    fn as_seq_at() {
        assert_throws!(
            sscanf!("abc: 1", "{SeqAtStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::SeqAtStruct -> as_seq() -> at(1) -> parse as should_panic::context::FailingStruct"#
        );

        assert_throws!(
            sscanf!("abc: 5", "{SeqAtStruct}").unwrap(),
            r#"sscanf: index 5 is out of bounds of a SeqMatch with 4 children.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::SeqAtStruct -> as_seq() -> at(5)"#
        );

        assert_throws!(
            sscanf!("abc: 0", "{SeqAtStruct}").unwrap(),
            r#"sscanf: sub-match at index 0 was not a Matcher.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::SeqAtStruct -> as_seq() -> at(0)"#
        );
    }

    #[test]
    fn as_seq_get() {
        struct SeqGetStruct(FailingStruct);
        impl FromScanf<'_> for SeqGetStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                Matcher::Seq(vec![
                    MatchPart::literal("a"),
                    FailingStruct::get_matcher(f).into(),
                ])
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches
                    .as_seq()
                    .get(1)
                    .unwrap()
                    .parse::<FailingStruct>(f)
                    .map(SeqGetStruct)
            }
        }
        assert_throws!(
            sscanf!("ab", "{SeqGetStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::as_seq_get::SeqGetStruct -> as_seq() -> get(1) -> parse as should_panic::context::FailingStruct"#
        );
    }

    #[test]
    fn as_seq_parse_at() {
        struct SeqParseAtStruct(FailingStruct);
        impl FromScanf<'_> for SeqParseAtStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                Matcher::Seq(vec![
                    MatchPart::literal("a"),
                    FailingStruct::get_matcher(f).into(),
                ])
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches
                    .as_seq()
                    .parse_at::<FailingStruct>(1, f)
                    .map(SeqParseAtStruct)
            }
        }
        assert_throws!(
            sscanf!("ab", "{SeqParseAtStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::as_seq_parse_at::SeqParseAtStruct -> as_seq() -> parse 1 as should_panic::context::FailingStruct"#
        );
    }

    #[test]
    fn as_seq_parse_field() {
        struct SeqParseFieldStruct(FailingStruct);
        impl FromScanf<'_> for SeqParseFieldStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                Matcher::Seq(vec![
                    MatchPart::literal("a"),
                    FailingStruct::get_matcher(f).into(),
                ])
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches
                    .as_seq()
                    .parse_field::<FailingStruct>("field_name", 1, f)
                    .map(SeqParseFieldStruct)
            }
        }
        assert_throws!(
            sscanf!("ab", "{SeqParseFieldStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::as_seq_parse_field::SeqParseFieldStruct -> as_seq() -> parse .field_name (index 1 as should_panic::context::FailingStruct)"#
        );
    }

    #[test]
    fn as_alt() {
        struct AltStruct;
        impl FromScanf<'_> for AltStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                Matcher::Alt(vec![
                    Matcher::from_regex("a").unwrap(),
                    FailingStruct::get_matcher(f),
                ])
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches
                    .as_alt()
                    .get()
                    .parse::<FailingStruct>(f)
                    .map(|_| AltStruct)
            }
        }

        assert_throws!(
            sscanf!("hi", "{AltStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::as_alt::AltStruct -> as_alt(1 matched) -> parse as should_panic::context::FailingStruct"#
        );
    }

    #[test]
    fn as_alt_enum() {
        struct AltStruct;
        impl FromScanf<'_> for AltStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                Matcher::Alt(vec![
                    Matcher::from_regex("a").unwrap(),
                    FailingStruct::get_matcher(f),
                ])
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches
                    .as_alt_enum(&["A", "B"])
                    .get()
                    .parse::<FailingStruct>(f)
                    .map(|_| AltStruct)
            }
        }

        assert_throws!(
            sscanf!("hi", "{AltStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::as_alt_enum::AltStruct -> as_alt(B matched) -> parse as should_panic::context::FailingStruct"#
        );
    }

    #[test]
    fn as_opt() {
        struct OptStruct;
        impl FromScanf<'_> for OptStruct {
            fn get_matcher(f: &FormatOptions) -> Matcher {
                FailingStruct::get_matcher(f).optional()
            }
            fn from_match_tree(matches: MatchTree<'_, '_>, f: &FormatOptions) -> Option<Self> {
                matches
                    .as_opt()
                    .unwrap()
                    .parse::<FailingStruct>(f)
                    .map(|_| OptStruct)
            }
        }

        assert_throws!(
            sscanf!("hi", "{OptStruct}").unwrap(),
            r#"sscanf: MatchTree::as_opt called on a Regex Match.
Context: sscanf -> as_seq() -> parse 0 as should_panic::context::as_opt::OptStruct -> as_opt() -> parse as should_panic::context::FailingStruct"#
        );
    }
}
