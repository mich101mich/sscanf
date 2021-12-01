use sscanf::*;

mod types {
    mod full_f32;
    mod full_f64;
    mod hex_number;
}

#[cfg(feature = "chrono")]
mod chrono;

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
fn config_numbers() {
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
#[should_panic(expected = "sscanf cannot generate Regex")]
fn invalid_regex_representation() {
    struct Test;
    impl std::str::FromStr for Test {
        type Err = ();
        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Test)
        }
    }
    impl RegexRepresentation for Test {
        const REGEX: &'static str = ")";
    }
    scanf!("hi", "{}", Test);
}

#[test]
fn custom_regex() {
    let input = "ab123cd";
    let parsed = scanf!(input, r"{}{/\d/}{/\d\d.*/}", String, u8, String);
    assert_eq!(parsed, Some((String::from("ab"), 1, String::from("23cd"))));

    let input = r"({(\}*[\{";
    let parsed = scanf!(input, r"{/\(\\\{\(\\\\\}\*/}{/\[\\\\\\\{/}", String, String);
    assert_eq!(parsed, Some((String::from("({(\\}*"), String::from("[\\{"))));
}

#[test]
fn custom_chrono_type() {
    #[derive(Debug, PartialEq)]
    struct DateTime(usize);

    impl sscanf::RegexRepresentation for DateTime {
        const REGEX: &'static str = r"\d+";
    }
    impl std::str::FromStr for DateTime {
        type Err = <usize as std::str::FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(DateTime(s.parse()?))
        }
    }

    let input = "42";
    let parsed = sscanf::scanf!(input, "{}", DateTime);
    assert_eq!(parsed, Some(DateTime(42)));
}

#[test]
fn failing_tests() {
    // Error Messages are different in nightly => Different .stderr files
    let nightly = rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;
    let channel = if nightly { "ch_nightly" } else { "ch_stable" };
    let os = if cfg!(windows) {
        "os_windows"
    } else {
        "os_linux"
    };
    let chrono = if cfg!(feature = "chrono") {
        "feat_chrono"
    } else {
        "feat_no_chrono"
    };

    let t = trybuild::TestCases::new();

    let path = std::path::PathBuf::from("tests/fail");
    run_fail_test(&t, &path);
    run_fail_test(&t, &path.join(channel));

    let os_path = path.join(os);
    run_fail_test(&t, &os_path);
    run_fail_test(&t, &os_path.join(channel));

    let chrono_path = path.join(chrono);
    run_fail_test(&t, &chrono_path);
    run_fail_test(&t, &chrono_path.join(channel));
}

fn run_fail_test(t: &trybuild::TestCases, path: &std::path::Path) {
    if path.exists() {
        t.compile_fail(path.join("*.rs").display().to_string());
    }
}
