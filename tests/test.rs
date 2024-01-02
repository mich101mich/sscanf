use sscanf::*;
use std::str::FromStr;

mod types {
    mod full_f32;
    mod full_f64;
    mod hex_number;
}

mod derive {
    mod r#enum;
    mod r#field;
    mod r#struct;
}

#[test]
fn basic() {
    let input = "Test 5 1.4 {} bob!";
    let output = sscanf!(input, "Test {usize} {f32} {{}} {}!", std::string::String);
    let (a, b, c) = output.unwrap();
    assert_eq!(a, 5);
    assert!((b - 1.4).abs() < f32::EPSILON, "b is {}", b);
    assert_eq!(c, "bob");

    let n = sscanf!(input, "hi");
    n.unwrap_err();

    let input = "Position<5,0.3,2>; Dir: N24E10";
    let output = sscanf!(
        input,
        "Position<{f32},{f32},{f32}>; Dir: {char}{usize}{char}{usize}",
    );
    assert_eq!(output.unwrap(), (5.0, 0.3, 2.0, 'N', 24, 'E', 10));

    let output = sscanf!("hi", "{str}").unwrap();
    assert_eq!(output, "hi");
}

#[test]
fn no_types() {
    let result = sscanf!("hi", "hi");
    result.unwrap();
    let result = sscanf!("hi", "no");
    result.unwrap_err();
}

#[test]
fn get_regex() {
    let input = "Test 5 {} bob!";
    let regex = sscanf_get_regex!("Test {usize} {{}} {}!", std::string::String);
    assert_eq!(regex.as_str(), r"^Test (\+?\d{1,20}) \{\} (.+?)!$");

    let output = regex.captures(input);
    assert!(output.is_some());
    let output = output.unwrap();
    assert_eq!(output.get(1).map(|m| m.as_str()), Some("5"));
    assert_eq!(output.get(2).map(|m| m.as_str()), Some("bob"));
}

#[test]
fn unescaped() {
    let input = "5.0SOME_RANDOM_TEXT3";
    let output = sscanf_unescaped!(input, "{f32}.*{usize}");
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
    let output = sscanf!(input, "{}", Bob<usize>);
    assert_eq!(output.unwrap(), Default::default());

    let input = "Test";
    let output = sscanf!(input, "{Bob<usize>}");
    assert_eq!(output.unwrap(), Default::default());
}

#[test]
fn config_numbers() {
    // example from lib.rs
    let input = "A Sentence with Spaces. Number formats: 0xab01 0o127 0b101010.";
    let parsed = sscanf!(input, "{str}. Number formats: {usize:x} {i32:o} {u8:b}.");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, "A Sentence with Spaces");
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);

    // negative numbers
    let input = "-10 -0xab01 -0o127 -0b101010";
    let parsed = sscanf!(input, "{i32:r3} {isize:x} {i32:o} {i8:b}");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, -3);
    assert_eq!(b, -0xab01);
    assert_eq!(c, -0o127);
    assert_eq!(d, -0b101010);

    let input = "-80 -0x80 7f 0x7f";
    let parsed = sscanf!(input, "{i8:x} {i8:x} {i8:x} {i8:x}");
    assert_eq!(parsed.unwrap(), (-128, -128, 127, 127)); // note that +128 would be out of range of i8

    // explicit positive numbers
    let input = "+10 +0xab01 +0o127 +0b101010";
    let parsed = sscanf!(input, "{i32:r3} {isize:x} {i32:o} {i8:b}");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, 3);
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);

    // negative number on unsigned
    let input = "-0xab01";
    let parsed = sscanf!(input, "{usize:x}");
    parsed.unwrap_err();

    // explicit positive number with prefix
    let input = "+10 +0xab01 +0o127 +0b101010";
    let parsed = sscanf!(input, "{u32:r3} {usize:x} {u32:o} {u8:b}");
    let (a, b, c, d) = parsed.unwrap();
    assert_eq!(a, 3);
    assert_eq!(b, 0xab01);
    assert_eq!(c, 0o127);
    assert_eq!(d, 0b101010);

    // forced and optional prefixes
    let prefix = "0xa1 0o17 0b101010";
    let no_prefix = "a1 17 101010";
    let out = (0xa1, 0o17, 0b101010);

    // :x etc have optional prefixes
    assert_eq!(out, sscanf!(prefix, "{u8:x} {u8:o} {u8:b}").unwrap());
    assert_eq!(out, sscanf!(no_prefix, "{u8:x} {u8:o} {u8:b}").unwrap());

    // :#x etc forces the prefix
    assert_eq!(out, sscanf!(prefix, "{u8:#x} {u8:#o} {u8:#b}").unwrap());
    sscanf!(no_prefix, "{u8:#x} {u8:#o} {u8:#b}").unwrap_err();

    // :r16 etc have no prefix
    sscanf!(prefix, "{u8:r16} {u8:r8} {u8:r2}").unwrap_err();
    assert_eq!(out, sscanf!(no_prefix, "{u8:r16} {u8:r8} {u8:r2}").unwrap());
}

#[test]
fn tuple_struct_reorder() {
    #[derive(Debug, PartialEq, FromScanf)]
    #[sscanf(format = "#{2:x}{1:x}{0:x}")]
    struct BGRColor(u8, u8, u8);

    let input = "#ff8811"; // rgb
    let parsed = sscanf!(input, "{}", BGRColor);
    assert_eq!(parsed.unwrap(), BGRColor(0x11, 0x88, 0xff));
}

#[test]
fn custom_regex() {
    let input = "ab123cd";
    let parsed = sscanf!(input, r"{str}{u8:/\d/}{str:/\d\d.*/}");
    assert_eq!(parsed.unwrap(), ("ab", 1, "23cd"));

    let input = r"({(\}*[\{";
    let parsed = sscanf!(input, r"{:/\(\{\(\\\}\*/}{:/\[\\\{/}", str, str);
    assert_eq!(parsed.unwrap(), (r"({(\}*", r"[\{"));

    #[derive(Debug, PartialEq)]
    struct NoRegex;
    impl FromStr for NoRegex {
        type Err = std::convert::Infallible;
        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            Ok(NoRegex)
        }
    }
    let parsed = sscanf!(input, "{NoRegex:/.*/}");
    assert_eq!(parsed.unwrap(), NoRegex);
}

#[test]
fn derived_from_str() {
    #[derive(Debug, PartialEq, FromScanf)]
    #[sscanf("{}: {}")]
    struct Bob {
        name: String,
        value: usize,
    }

    let expected = Bob {
        name: "bob".to_string(),
        value: 5,
    };

    assert_eq!(Bob::from_str("bob: 5").unwrap(), expected);
    assert!(Bob::from_str("bob: 6").unwrap() != expected);
    assert!(Bob::from_str("bob : 5").unwrap() != expected);

    assert!(Bob::from_str("{bob: 5}").is_err());
    assert!(Bob::from_str("bob: a").is_err());
    assert!(Bob::from_str("bob").is_err());
}

#[test]
fn string_lifetime() {
    // compare with tests/fail/str_lifetime.rs
    let s;
    {
        let input = String::from("hi");
        s = sscanf!(input, "{String}").unwrap();
    }
    println!("{}", s);

    // check if sscanf works with various function signatures
    fn process(a: &str) -> &str {
        sscanf!(a, "{str}").unwrap()
    }
    process("hi");

    fn process_with_borrow(a: &str) -> &str {
        sscanf!(a, "{&str}").unwrap()
    }
    process_with_borrow("hi");

    fn process_with_lifetime<'a, 'b>(_a: &'a str, b: &'b str) -> &'b str {
        sscanf!(b, "{&str}").unwrap()
    }
    process_with_lifetime("hi", "hi");

    fn process_cow<'a>(a: &'a str) -> std::borrow::Cow<'a, str> {
        sscanf!(a, "{Cow<str>}").unwrap()
    }
    process_cow("hi");
}

#[test]
fn error_lifetime() {
    fn foo() -> Result<(), Box<dyn std::error::Error>> {
        let input = String::from("hi");
        sscanf!(input, "{String}").map_err(|err| err.to_string())?;
        Ok(())
    }
    foo().unwrap();
}

#[test]
#[ignore]
fn error_message_tests() {
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
