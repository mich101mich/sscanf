#![allow(deprecated)]
use sscanf::*;

#[test]
fn hex_number() {
    let input = "0xaAbBcCdDeEfF";
    let output = sscanf!(input, "{}", HexNumber).unwrap();
    assert_eq!(output, 0xaabbccddeeff);

    let input = "0x1234567890";
    let output = sscanf!(input, "{}", HexNumber).unwrap();
    assert_eq!(output, 0x1234567890);

    let input = "10x20XA";
    let output = sscanf!(input, "{}{}{}", HexNumber, HexNumber, HexNumber).unwrap();
    assert_eq!(output.0, 1);
    assert_eq!(output.1, 2);
    assert_eq!(output.2, 0xa);
}
