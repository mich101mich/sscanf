use sscanf::*;

#[test]
fn basic() {
    #[derive(FromScanf, Debug, PartialEq)]
    enum Number {
        #[sscanf(format = "0")]
        Zero,
        #[sscanf(format = "{}")]
        Whole(isize),
        #[sscanf(format = "{numerator}/{denominator}")]
        Fraction {
            numerator: isize,
            denominator: usize,
        },
    }

    let input = "0 5 -1/2.";
    let (zero, whole, fraction) = sscanf!(input, "{Number} {Number} {Number}.").unwrap();

    assert_eq!(zero, Number::Zero);

    assert_eq!(whole, Number::Whole(5));

    assert_eq!(
        fraction,
        Number::Fraction {
            numerator: -1,
            denominator: 2
        }
    );
}

#[test]
fn order() {
    #[derive(FromScanf, Debug, PartialEq)]
    enum Number {
        #[sscanf(format = "{}")]
        Whole(isize),
        #[sscanf(format = "0")]
        Zero,
        #[sscanf(format = "{}/{}")]
        Fraction(isize, usize),
    }

    let input = "0 5 -1/2.";
    let (zero, whole, fraction) = sscanf!(input, "{Number} {Number} {Number}.").unwrap();

    assert_eq!(zero, Number::Whole(0));
    assert_eq!(whole, Number::Whole(5));
    assert_eq!(fraction, Number::Fraction(-1, 2));
}

#[test]
fn not_constructible() {
    #[allow(dead_code)]
    #[derive(FromScanf, Debug, PartialEq)]
    enum Number {
        #[sscanf(format = "0")]
        Zero,
        Whole(isize),
        Fraction(isize, usize),
    }

    assert_eq!(sscanf!("0", "{Number}").unwrap(), Number::Zero);
    assert!(sscanf!("5", "{Number}").is_err());
    assert!(sscanf!("-1/2", "{Number}").is_err());
}

#[test]
fn transparent() {
    #[derive(FromScanf, Debug, PartialEq)]
    enum Dynamic {
        #[sscanf(transparent)]
        Number(isize),
        #[sscanf(transparent)]
        String(String),
    }

    let input = "123 hello 0 hi";
    let (num, string, zero, hi) =
        sscanf!(input, "{Dynamic} {Dynamic} {Dynamic} {Dynamic}").unwrap();
    assert_eq!(num, Dynamic::Number(123));
    assert_eq!(string, Dynamic::String("hello".to_string()));
    assert_eq!(zero, Dynamic::Number(0));
    assert_eq!(hi, Dynamic::String("hi".to_string()));
}

#[test]
fn autogen() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(autogen)]
    enum Words {
        Hello,
        World,
        Hi,
    }

    let input = "Hello World Hi";
    let expected = (Words::Hello, Words::World, Words::Hi);
    let parsed = sscanf!(input, "{Words} {Words} {Words}").unwrap();
    assert_eq!(parsed, expected);

    let input_lower = "hello world hi";
    sscanf!(input_lower, "{Words} {Words} {Words}").unwrap_err();

    #[derive(FromScanf, Debug, PartialEq)]
    #[allow(dead_code)]
    #[sscanf(autogen)]
    enum WordsWithFields {
        Hello,
        #[sscanf(skip)]
        World(usize),
        #[sscanf(transparent)]
        Hi(u8),
    }

    let input = "Hello 5";
    let expected = (WordsWithFields::Hello, WordsWithFields::Hi(5));
    let parsed = sscanf!(input, "{WordsWithFields} {WordsWithFields}").unwrap();
    assert_eq!(parsed, expected);

    let input_world = "World";
    sscanf!(input_world, "{WordsWithFields}").unwrap_err();
}

#[test]
fn autogen_cases() {
    let cases: std::collections::HashMap<_, _> = [
        ("lowercase", "helloworld"),
        ("UPPERCASE", "HELLOWORLD"),
        ("lower case", "hello world"),
        ("UPPER CASE", "HELLO WORLD"),
        ("PascalCase", "HelloWorld"),
        ("camelCase", "helloWorld"),
        ("snake_case", "hello_world"),
        ("SCREAMING_SNAKE_CASE", "HELLO_WORLD"),
        ("kebab-case", "hello-world"),
        ("SCREAMING-KEBAB-CASE", "HELLO-WORLD"),
        ("rANdOmCasE", "hElLowOrLd"),
    ]
    .iter()
    .copied()
    .collect();

    let mut errors = String::new();

    macro_rules! run_check {
        ($case: literal : $($accepted: literal),+) => {{
            #[derive(FromScanf, Debug, PartialEq)]
            #[sscanf(autogen = $case)]
            enum Word {
                HelloWorld
            }

            let mut accepted = std::collections::HashSet::new();
            $( accepted.insert($accepted); )+

            for (name, input) in &cases {
                let result = sscanf!(input, "{Word}");
                if accepted.contains(name) {
                    if result.is_err() {
                        errors.push_str(&format!(r#"input "{}" should match autogen="{}""#, input, $case));
                    }
                } else {
                    if result.is_ok() {
                        errors.push_str(&format!(r#"input "{}" incorrectly matched autogen="{}""#, input, $case));
                    }
                }
            }
        }};
    }

    run_check!("CaseSensitive": "PascalCase");
    run_check!("CAsEiNsenSiTIvE": "PascalCase", "lowercase", "UPPERCASE", "PascalCase", "camelCase", "rANdOmCasE");
    run_check!("lowercase": "lowercase");
    run_check!("UPPERCASE": "UPPERCASE");
    run_check!("lower case": "lower case");
    run_check!("UPPER CASE": "UPPER CASE");
    run_check!("PascalCase": "PascalCase");
    run_check!("camelCase": "camelCase");
    run_check!("snake_case": "snake_case");
    run_check!("SCREAMING_SNAKE_CASE": "SCREAMING_SNAKE_CASE");
    run_check!("kebab-case": "kebab-case");
    run_check!("SCREAMING-KEBAB-CASE": "SCREAMING-KEBAB-CASE");

    assert!(errors.is_empty(), "{}", errors);
}
