use sscanf::*;

static CORRECT_INPUT: &str = "Testing with (3.4,1,-2,0)!";
macro_rules! correct_result {
    (named) => {
        TestStruct {
            a: 0,
            b: String::from("1"),
            c: -2,
            d: 3.4,
        }
    };
    (unnamed) => {
        TestStruct(0, String::from("1"), -2, 3.4)
    };
}

static WRONG_INPUTS: &[&str] = &[
    "Testing with (3.4,1,-2)!",
    "Testing with (3.-4,1,-2,0)!",
    "Testing with (3.4,,-2,0)!",
    "Testing with (3.4,1,,0)!",
    "Testing with (3.4,1,+-2,0)!",
    "Testing with (3.4,1,-2,1000)!",
];

#[test]
fn basic() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({d},{b},{c},{a})")]
    struct TestStruct {
        a: u8,
        b: String,
        c: isize,
        d: f32,
    }

    let ret = sscanf!(CORRECT_INPUT, "Testing with {TestStruct}!").unwrap();
    assert_eq!(ret, correct_result!(named));

    for input in WRONG_INPUTS {
        let res = sscanf!(input, "Testing with {TestStruct}!");
        res.unwrap_err();
    }
}

#[test]
fn indexed() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({3},{1},{2},{0})")]
    struct TestStruct {
        a: u8,
        b: String,
        c: isize,
        d: f32,
    }

    let ret = sscanf!(CORRECT_INPUT, "Testing with {TestStruct}!").unwrap();
    assert_eq!(ret, correct_result!(named));

    for input in WRONG_INPUTS {
        let res = sscanf!(input, "Testing with {TestStruct}!");
        res.unwrap_err();
    }
}

#[test]
fn auto_indexed() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({},{},{},{})")]
    struct TestStruct {
        d: f32,
        b: String,
        c: isize,
        a: u8,
    }

    let ret = sscanf!(CORRECT_INPUT, "Testing with {TestStruct}!").unwrap();
    assert_eq!(ret, correct_result!(named));

    for input in WRONG_INPUTS {
        let res = sscanf!(input, "Testing with {TestStruct}!");
        res.unwrap_err();
    }
}

#[test]
fn tuple_struct() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({3},{1},{2},{0})")]
    struct TestStruct(u8, String, isize, f32);

    let ret = sscanf!(CORRECT_INPUT, "Testing with {TestStruct}!").unwrap();
    assert_eq!(ret, correct_result!(unnamed));

    for input in WRONG_INPUTS {
        let res = sscanf!(input, "Testing with {TestStruct}!");
        res.unwrap_err();
    }
}

#[test]
fn transparent() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(transparent)]
    struct TestStructTuple(usize);

    let ret = sscanf!("5", "{TestStructTuple}").unwrap();
    assert_eq!(ret, TestStructTuple(5));

    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(transparent)]
    struct TestStructNamed {
        a: usize,
    }

    let ret = sscanf!("5", "{TestStructNamed}").unwrap();
    assert_eq!(ret, TestStructNamed { a: 5 });
}
