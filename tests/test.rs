use sscanf::*;
use std::str::FromStr;

mod types {
    mod full_f32;
    mod full_f64;
    mod hex_number;
}

use sscanf::RegexRepresentation;

#[derive(FromScanf, Debug, PartialEq)]
#[scanf(format = "({name},{age})")]
struct Person {
    name: String,
    age: u8,
}

#[test]
fn test_from_regex() {
    let input = "Hi, I'm (Bob,42)!";
    let bob = scanf!(input, "Hi, I'm {Person}!").unwrap();

    assert_eq!(
        bob,
        Person {
            name: "Bob".to_string(),
            age: 42
        }
    );
}

// #[derive(FromScanf, Debug, PartialEq)]
// #[scanf(Whole = "{0}", Fraction = "{numerator}/{denominator}")]
// enum Number {
//     Whole(isize),
//     Fraction {
//         numerator: isize,
//         denominator: isize,
//     },
// }

// #[test]
// fn test_from_regex() {
//     let input = "Hi, I'm (Bob,42)!. I have 5 dollars and -1/2 pennies.";
//     let (bob, dollars, pennies) = sscanf::scanf!(
//         input,
//         "Hi, I'm {Person}! I have {Number} dollars and {Number} pennies."
//     )
//     .unwrap();

//     assert_eq!(
//         bob,
//         Person {
//             name: "Bob".to_string(),
//             age: 42
//         }
//     );

//     assert_eq!(dollars, Number::Whole(5));

//     assert_eq!(pennies, Number::Fraction(-1, 2));
// }

#[test]
fn basic() {
    let input = "Test 5 1.4 {} bob!";
    let output = scanf!(input, "Test {usize} {f32} {{}} {}!", std::string::String);
    assert!(output.is_ok());
    let (a, b, c) = output.unwrap();
    assert_eq!(a, 5);
    assert!((b - 1.4).abs() < f32::EPSILON, "b is {}", b);
    assert_eq!(c, "bob");

    let n = scanf!(input, "hi");
    assert!(n.is_err());

    let input = "Position<5,0.3,2>; Dir: N24E10";
    let output = scanf!(
        input,
        "Position<{f32},{f32},{f32}>; Dir: {char}{usize}{char}{usize}",
    );
    assert_eq!(output.unwrap(), (5.0, 0.3, 2.0, 'N', 24, 'E', 10));

    let output = scanf!("hi", "{str}").unwrap();
    assert_eq!(output, "hi");
}

#[test]
fn no_types() {
    let result = scanf!("hi", "hi");
    assert_eq!(result.unwrap(), ());
    let result = scanf!("hi", "no");
    assert!(result.is_err());
}

#[test]
fn alternate_inputs() {
    assert_eq!(scanf!("5", "{usize}").unwrap(), 5);

    let input = "5";
    assert_eq!(scanf!(input, "{usize}").unwrap(), 5);

    let input = String::from("5");
    assert_eq!(scanf!(input, "{usize}").unwrap(), 5);

    // These don't work because of the lifetime
    // let input = String::from("5");
    // assert_eq!(scanf!(input.as_str(), "{usize}"), Ok(5));

    // let input = ['5'];
    // assert_eq!(scanf!(input.iter().collect::<String>(), "{usize}"), Ok(5));
}

#[test]
fn get_regex() {
    let input = "Test 5 1.4 {} bob!";
    let regex = scanf_get_regex!("Test {usize} {f32} {{}} {}!", std::string::String);
    assert_eq!(
        regex.as_str(),
        r"^Test (\+?\d{1,20}) ([-+]?\d+\.?\d*) \{\} (.+?)!$"
    );

    let output = regex.captures(input);
    assert!(output.is_some());
    let output = output.unwrap();
    assert_eq!(output.get(1).map(|m| m.as_str()), Some("5"));
    assert_eq!(output.get(2).map(|m| m.as_str()), Some("1.4"));
    assert_eq!(output.get(3).map(|m| m.as_str()), Some("bob"));
}

#[test]
fn unescaped() {
    let input = "5.0SOME_RANDOM_TEXT3";
    let output = scanf_unescaped!(input, "{f32}.*{usize}");
    assert_eq!(output.unwrap(), (5.0, 3));
}

#[test]
fn generic_types() {
    #[derive(Debug, PartialEq, Eq, Default)]
    pub struct Bob<T>(pub std::marker::PhantomData<T>);
    impl<T> RegexRepresentation for Bob<T> {
        const REGEX: &'static str = ".*";
    }
    impl<T: Default> FromStr for Bob<T> {
        type Err = <f64 as FromStr>::Err;
        fn from_str(_: &str) -> Result<Self, Self::Err> {
            Ok(Default::default())
        }
    }

    let input = "Test";
    let output = scanf!(input, "{}", Bob<usize>);
    assert_eq!(output.unwrap(), Default::default());

    let input = "Test";
    let output = scanf!(input, "{Bob<usize>}");
    assert_eq!(output.unwrap(), Default::default());
}

#[test]
fn config_numbers() {
    let input = "A Sentence with Spaces. Number formats: 0xab01 0o127 0b101010.";
    let parsed = scanf!(input, "{str}. Number formats: {usize:x} {i32:o} {u8:b}.");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, "A Sentence with Spaces");
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);

    let input = "-10 -0xab01 -0o127 -0b101010";
    let parsed = scanf!(input, "{i32:r3} {isize:x} {i32:o} {i8:b}");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, -3);
    assert_eq!(b, -0xab01);
    assert_eq!(c, -0o127);
    assert_eq!(d, -0b101010);

    let input = "+10 +0xab01 +0o127 +0b101010";
    let parsed = scanf!(input, "{i32:r3} {isize:x} {i32:o} {i8:b}");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, 3);
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);

    let input = "-0xab01";
    let parsed = scanf!(input, "{usize:x}");
    assert!(parsed.is_err());

    let input = "+10 +0xab01 +0o127 +0b101010";
    let parsed = scanf!(input, "{u32:r3} {usize:x} {u32:o} {u8:b}");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, 3);
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);
}

#[test]
#[should_panic(expected = "scanf: Cannot generate Regex")]
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
    scanf!("hi", "{Test}").unwrap();
}

#[test]
fn custom_regex() {
    let input = "ab123cd";
    let parsed = scanf!(input, r"{str}{u8:/\d/}{str:/\d\d.*/}");
    assert_eq!(parsed.unwrap(), ("ab", 1, "23cd"));

    let input = r"({(\}*[\{";
    let parsed = scanf!(input, r"{:/\(\{\(\\\}\*/}{:/\[\\\{/}", str, str);
    assert_eq!(parsed.unwrap(), ("({(\\}*", "[\\{"));

    #[derive(Debug, PartialEq)]
    struct NoRegex;
    impl FromStr for NoRegex {
        type Err = std::convert::Infallible;
        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            Ok(NoRegex)
        }
    }
    let parsed = scanf!(input, "{NoRegex:/.*/}");
    assert_eq!(parsed.unwrap(), NoRegex);
}

#[test]
#[should_panic(expected = "ScanfMatchFailed")]
fn check_error_regex() {
    scanf!("hi", "bob").unwrap();
}
#[test]
#[should_panic(
    expected = "FromStrFailedError { type_name: \"usize\", error: ParseIntError { kind: InvalidDigit } }"
)]
fn check_error_from_str_1() {
    scanf!("5bobhibob", "{u32}bob{usize:/.*/}bob").unwrap();
}
#[test]
#[should_panic(
    expected = "FromStrFailedError { type_name: \"test::check_error_from_str_2::Test\", error: ParseIntError { kind: InvalidDigit } }"
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
    scanf!("bobhibob", "bob{}bob", Test).unwrap();
}

#[test]
fn string_lifetime() {
    // compare with tests/fail/str_lifetime.rs
    let s;
    {
        let input = String::from("hi");
        s = sscanf::scanf!(input, "{String}").unwrap();
    }
    println!("{}", s);

    // check if scanf works with this function signature
    fn _process(a: &str) -> &str {
        scanf!(a, "{str}").unwrap()
    }
}

#[test]
fn error_lifetime() {
    fn foo() -> Result<(), Box<dyn std::error::Error>> {
        let input = String::from("hi");
        sscanf::scanf!(input, "{String}").map_err(|err| err.to_string())?;
        Ok(())
    }
    foo().unwrap();
}

#[test]
#[should_panic(expected = "scanf: Regex has 3 capture groups, but 2 were expected.
If you use ( ) in a custom Regex, please add a '?:' at the beginning to avoid
forming a capture group like this:
    ...(...)...  =>  ...(?:...)...
")]
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
    scanf!("5", "{Test}").unwrap();
}

#[test]
fn failing_tests() {
    let root = std::path::PathBuf::from("tests/fail");
    let mut paths = vec![root.clone()];

    // Error Messages are different in nightly => Different .stderr files
    let nightly = rustc_version::version_meta().unwrap().channel == rustc_version::Channel::Nightly;
    let channel = if nightly { "nightly" } else { "stable" };
    paths.push(root.join(channel));

    let t = trybuild::TestCases::new();
    for mut path in paths {
        path.push("*.rs");
        t.compile_fail(path.display().to_string());
    }
}
