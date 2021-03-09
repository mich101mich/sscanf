use sscanf::*;

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
    assert_eq!(b, 1.4);
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
fn get_regex() {
    let input = "Test 5 1.4 {} bob!";
    let regex = scanf_get_regex!("Test {} {} {{}} {}!", usize, f32, std::string::String);
    assert_eq!(
        regex.as_str(),
        r"^Test (?P<type_1>\+?\d+) (?P<type_2>[-+]?\d+\.?\d*) \{\} (?P<type_3>.+)!$"
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
fn failing_tests() {
    trybuild::TestCases::new().compile_fail("tests/fail/*.rs");

    // Error Messages are better in nightly => Different .stderr files
    if rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly {
        trybuild::TestCases::new().compile_fail("tests/fail_nightly/*.rs");
    } else {
        trybuild::TestCases::new().compile_fail("tests/fail_stable/*.rs");
    }
}
