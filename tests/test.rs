use sscanf::*;

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
    assert!((b - 1.4).abs() < f32::EPSILON, "b is {b}");
    assert_eq!(c, "bob");

    let n = sscanf!(input, "hi");
    assert!(n.is_none());

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
    assert!(result.is_none());
}

#[test]
fn regex_format_string() {
    let input = "5.0SOME_RANDOM_TEXT3";
    let output = sscanf_with_regex!(input, "{f32}.*{usize}");
    assert_eq!(output.unwrap(), (5.0, 3));
}

#[test]
fn sscanf_parser() {
    let mut parser = sscanf_parser!("Employee #{usize}!");

    let output = parser.parse("Employee #42!").unwrap();
    assert_eq!(output, 42);

    let output = parser.parse("Employee #7!").unwrap();
    assert_eq!(output, 7);

    assert_eq!(parser.parse("Invalid Input"), None);
    assert_eq!(parser.parse("Employee #X"), None);
}

#[test]
fn generic_types() {
    #[derive(Debug, PartialEq, Eq, Default)]
    pub struct Bob<T>(pub std::marker::PhantomData<T>);
    impl<T: Default> FromScanfSimple<'_> for Bob<T> {
        const REGEX: &'static str = ".*";
        fn from_match(_: &str) -> Option<Self> {
            Some(Default::default())
        }
    }

    let input = "Test";
    let output = sscanf!(input, "{}", Bob<usize>);
    assert_eq!(output.unwrap(), Default::default());

    let input = "Test";
    let output = sscanf!(input, "{Bob<usize>}");
    assert_eq!(output.unwrap(), Default::default());

    fn parse_quoted<'input, T: FromScanf<'input>>(s: &'input str) -> Option<T> {
        sscanf!(s, "\"{T}\"")
    }

    let input = r#""Hello, World!""#;
    let output: &str = parse_quoted(input).unwrap();
    assert_eq!(output, "Hello, World!");

    let input = r#""42""#;
    let output = parse_quoted::<usize>(input).unwrap();
    assert_eq!(output, 42);
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
    assert!(parsed.is_none());

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
    assert!(sscanf!(no_prefix, "{u8:#x} {u8:#o} {u8:#b}").is_none());

    // :r16 etc have no prefix
    assert!(sscanf!(prefix, "{u8:r16} {u8:r8} {u8:r2}").is_none());
    assert_eq!(out, sscanf!(no_prefix, "{u8:r16} {u8:r8} {u8:r2}").unwrap());

    // using type aliases
    type MyNumber = usize;
    let input = "0xab01";
    let parsed = sscanf!(input, "{MyNumber:x}");
    assert_eq!(parsed.unwrap(), 0xab01);

    // regex overrides
    let input = "123";
    assert_eq!(sscanf!(input, "{usize:/\\d{3}/}").unwrap(), 123);
    assert_eq!(sscanf!(input, "{usize:x /\\d{3}/}").unwrap(), 0x123);
    assert_eq!(sscanf!(input, "{usize:o /\\d{3}/}").unwrap(), 0o123);
    assert_eq!(sscanf!(input, "{usize:r36 /\\d{3}/}").unwrap(), 1371);

    assert!(sscanf!(input, "{usize:#b /\\d{3}/}").is_none());
    assert!(sscanf!(input, "{usize:#o /\\d{3}/}").is_none());
    assert!(sscanf!(input, "{usize:#x /\\d{3}/}").is_none());

    let input = "0x123";
    assert_eq!(sscanf!(input, "{usize:x /.*/}").unwrap(), 0x123);
    assert_eq!(sscanf!(input, "{usize:#x /.*/}").unwrap(), 0x123);
    assert!(sscanf!(input, "{usize:/.*/}").is_none());

    assert_eq!(sscanf!("0b1010", "{usize:b /.*/}").unwrap(), 0b1010);
    assert_eq!(sscanf!("0o17", "{usize:o /.*/}").unwrap(), 0o17);
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
}

#[test]
fn custom_format_option() {
    #[derive(Debug, PartialEq)]
    struct MyOption<T>(Option<T>);

    impl<'input, T: FromScanf<'input>> FromScanf<'input> for MyOption<T> {
        fn get_matcher(options: &advanced::FormatOptions) -> advanced::Matcher {
            let mut format = options.clone();
            let option = format.custom.take().unwrap();
            let (prefix, suffix) = option.split_once("{}").unwrap();
            advanced::Matcher::Seq(vec![
                advanced::MatchPart::literal(prefix.to_string()),
                T::get_matcher(&format).into(),
                advanced::MatchPart::literal(suffix.to_string()),
            ])
            .optional()
        }

        fn from_match(
            matches: advanced::Match<'_, 'input>,
            options: &advanced::FormatOptions,
        ) -> Option<Self> {
            let inner = if let Some(m) = matches.as_opt() {
                Some(m.as_seq().parse_field("0", 1, options)?)
            } else {
                None
            };
            Some(Self(inner))
        }
    }

    let input = "Find the code '<12345>' in the text.";
    let parsed = sscanf!(
        input,
        "Find the code '{MyOption<usize>:[<{}>]}' in the text."
    )
    .unwrap();
    assert_eq!(parsed, MyOption(Some(12345)));

    let input = "Find the code '' in the text.";
    let parsed = sscanf!(
        input,
        "Find the code '{MyOption<usize>:[<{}>]}' in the text."
    )
    .unwrap();
    assert_eq!(parsed, MyOption(None));
}

#[test]
fn string_lifetime() {
    // compare with tests/fail/str_lifetime.rs
    let s;
    {
        let input = String::from("hi");
        s = sscanf!(input, "{String}").unwrap();
    }
    println!("{s}");

    // check if sscanf works with various function signatures
    fn process(a: &str) -> &str {
        sscanf!(a, "{str}").unwrap()
    }
    process("hi");

    fn process_with_borrow(a: &str) -> &str {
        sscanf!(a, "{&str}").unwrap()
    }
    process_with_borrow("hi");

    #[allow(clippy::needless_lifetimes)]
    fn process_with_lifetime<'a, 'b>(_a: &'a str, b: &'b str) -> &'b str {
        sscanf!(b, "{&str}").unwrap()
    }
    process_with_lifetime("hi", "hi");

    fn process_cow<'a>(a: &'a str) -> std::borrow::Cow<'a, str> {
        sscanf!(a, "{std::borrow::Cow<str>}").unwrap()
    }
    process_cow("hi");
}

#[test]
fn respects_raw_strings() {
    let input = "0  \\  \"  \n  \x41  \u{0041}";
    let parsed = sscanf!(input, "{usize}  \\  \"  \n  \x41  \u{0041}");
    assert_eq!(parsed.unwrap(), 0);

    let parsed = sscanf!(
        input,
        r#"{usize}  \  "  
  A  A"#
    );
    assert_eq!(parsed.unwrap(), 0);
}

#[test]
#[ignore]
fn error_message_tests() {
    err_span_check::run_on_fail_dir();
}
