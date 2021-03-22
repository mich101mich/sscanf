use sscanf::*;

#[test]
fn full_f64() {
    let input = "-Nan";
    let output = scanf!(input, "{}", FullF64).unwrap();
    assert!(output.is_nan());

    let input = "nanNaN-Naninf-inf";
    let output = scanf!(
        input,
        "{}{}{}{}{}",
        FullF64,
        FullF64,
        FullF64,
        FullF64,
        FullF64
    )
    .unwrap();
    assert!(output.0.is_nan());
    assert!(output.1.is_nan());
    assert!(output.2.is_nan());
    assert_eq!(*output.3, std::f64::INFINITY);
    assert_eq!(*output.4, std::f64::NEG_INFINITY);

    let output = scanf!("-2.0e4 2.0e4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, (FullF64(-2.0e4), FullF64(2.0e4)));

    let output = scanf!("-.1e-4 .1e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, (FullF64(-0.1e-4), FullF64(0.1e-4)));

    let expected = (FullF64(-2.0e-4), FullF64(2.0e-4));
    let output = scanf!("-2e-4 2e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, expected);

    let output = scanf!("-2.e-4 2.e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, expected);

    let output = scanf!("-2.0e-4 2.0e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, expected);
}
