#![allow(deprecated)]
use sscanf::*;

#[test]
fn hex_number() {
    let input = "0xaAbBcC"; // split because HexNumber uses usize which might be 32 or 64 bits
    let output = sscanf!(input, "{}", HexNumber).unwrap();
    assert_eq!(output, 0xaabbcc);

    let input = "0xdDeEfF";
    let output = sscanf!(input, "{}", HexNumber).unwrap();
    assert_eq!(output, 0xddeeff);

    let input = "0x12345";
    let output = sscanf!(input, "{}", HexNumber).unwrap();
    assert_eq!(output, 0x12345);

    let input = "0x67890";
    let output = sscanf!(input, "{}", HexNumber).unwrap();
    assert_eq!(output, 0x67890);

    let input = "10x20XA";
    let output = sscanf!(input, "{}{}{}", HexNumber, HexNumber, HexNumber).unwrap();
    assert_eq!(output.0, 1);
    assert_eq!(output.1, 2);
    assert_eq!(output.2, 0xa);
}
