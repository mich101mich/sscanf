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
    #[sscanf(autogen = "CaseSensitive")]
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
}
