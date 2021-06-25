use sscanf::*;

mod types {
    mod full_f32;
    mod full_f64;
    mod hex_number;
}

#[test]
fn basic() {
    let input = "Test 5 1.4 {} bob!";
    let output = scanf!(
        input,
        "Test {} {} {{}} {}!",
        usize,
        f32,
        std::string::String
    );
    assert!(output.is_some());
    let (a, b, c) = output.unwrap();
    assert_eq!(a, 5);
    assert!((b - 1.4).abs() < f32::EPSILON, "b is {}", b);
    assert_eq!(c, "bob");

    let n = scanf!(input, "hi");
    assert_eq!(n, None);

    let input = "Position<5,0.3,2>; Dir: N24E10";
    let output = scanf!(
        input,
        "Position<{},{},{}>; Dir: {}{}{}{}",
        f32,
        f32,
        f32,
        char,
        usize,
        char,
        usize
    );
    assert_eq!(output, Some((5.0, 0.3, 2.0, 'N', 24, 'E', 10)));
}

#[test]
fn no_types() {
    let result = scanf!("hi", "hi");
    assert_eq!(result, Some(()));
    let result = scanf!("hi", "no");
    assert_eq!(result, None);
}

#[test]
fn alternate_inputs() {
    assert_eq!(scanf!("5", "{}", usize), Some(5));

    let input = "5";
    assert_eq!(scanf!(input, "{}", usize), Some(5));

    let input = String::from("5");
    assert_eq!(scanf!(input, "{}", usize), Some(5));

    let input = String::from("5");
    assert_eq!(scanf!(input.as_str(), "{}", usize), Some(5));

    let input = ['5'];
    assert_eq!(
        scanf!(input.iter().collect::<String>(), "{}", usize),
        Some(5)
    );
}

#[test]
fn get_regex() {
    let input = "Test 5 1.4 {} bob!";
    let regex = scanf_get_regex!("Test {} {} {{}} {}!", usize, f32, std::string::String);
    assert_eq!(
        regex.as_str(),
        r"^Test (?P<type_1>\+?\d{1,20}) (?P<type_2>[-+]?\d+\.?\d*) \{\} (?P<type_3>.+)!$"
    );

    let output = regex.captures(input);
    assert!(output.is_some());
    let output = output.unwrap();
    assert_eq!(output.name("type_1").map(|m| m.as_str()), Some("5"));
    assert_eq!(output.name("type_2").map(|m| m.as_str()), Some("1.4"));
    assert_eq!(output.name("type_3").map(|m| m.as_str()), Some("bob"));
    assert_eq!(output.get(1).map(|m| m.as_str()), Some("5"));
    assert_eq!(output.get(2).map(|m| m.as_str()), Some("1.4"));
    assert_eq!(output.get(3).map(|m| m.as_str()), Some("bob"));
}

#[test]
fn unescaped() {
    let input = "5.0SOME_RANDOM_TEXT3";
    let output = scanf_unescaped!(input, "{}.*{}", f32, usize);
    assert_eq!(output, Some((5.0, 3)));
}

#[test]
fn generic_types() {
    #[derive(Debug, PartialEq, Default)]
    pub struct Bob<T>(pub std::marker::PhantomData<T>);
    impl<T> RegexRepresentation for Bob<T> {
        const REGEX: &'static str = ".*";
    }
    impl<T: Default> std::str::FromStr for Bob<T> {
        type Err = <f64 as std::str::FromStr>::Err;
        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Default::default())
        }
    }

    let input = "Test";
    let output = scanf!(input, "{}", Bob<usize>);
    assert_eq!(output, Some(Default::default()));
}

#[test]
fn config() {
    let input = "A Sentence with Spaces. Number formats: 0xab01 0o127 0b101010.";
    let parsed = scanf!(
        input,
        "{}. Number formats: {x} {o} {b}.",
        String,
        usize,
        i32,
        u8
    );
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, "A Sentence with Spaces");
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);
}

#[test]
fn failing_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/*.rs");
    t.compile_fail("tests/fail_syntax/*.rs");
    // Error Messages are better in nightly => Different .stderr files
    if rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly {
        t.compile_fail("tests/fail_nightly/*.rs");
    } else {
        t.compile_fail("tests/fail_stable/*.rs");
    }
}
