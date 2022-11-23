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
