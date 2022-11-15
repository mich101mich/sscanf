#![allow(deprecated)]
use sscanf::*;

#[test]
fn full_f64() {
    let input = "-Nan";
    let output = sscanf!(input, "{}", FullF64).unwrap();
    assert!(output.is_nan());

    let input = "nanNaN-Naninf-inf";
    let output = sscanf!(
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
    assert!(output.3.is_infinite());
    assert!(output.4.is_infinite());
    assert!(*output.3 > 0.0);
    assert!(*output.4 < 0.0);

    let output = sscanf!("-2.0e4 2.0e4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, (FullF64(-2.0e4), FullF64(2.0e4)));

    let output = sscanf!("-.1e-4 .1e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, (FullF64(-0.1e-4), FullF64(0.1e-4)));

    let expected = (FullF64(-2.0e-4), FullF64(2.0e-4));
    let output = sscanf!("-2e-4 2e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, expected);

    let output = sscanf!("-2.e-4 2.e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, expected);

    let output = sscanf!("-2.0e-4 2.0e-4", "{} {}", FullF64, FullF64).unwrap();
    assert_eq!(output, expected);
}
