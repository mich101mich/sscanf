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
fn defaults() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({d},{b},{c},{a})")]
    struct TestStruct {
        a: u8,
        b: String,
        #[sscanf(default)]
        e: Vec<usize>,
        #[sscanf(default = vec![5])]
        f: Vec<usize>,
        #[sscanf(default = outer_source())]
        g: Vec<usize>,
        #[sscanf(default = {
            let mut v = Vec::new();
            v.push(0);
            for _ in 0..7 {
                *v.last_mut().unwrap() +=1;
            }
            v
        })]
        h: Vec<usize>,
        c: isize,
        d: f32,
    }

    fn outer_source() -> Vec<usize> {
        vec![6]
    }

    let correct = TestStruct {
        a: 0,
        b: String::from("1"),
        c: -2,
        d: 3.4,
        e: Default::default(),
        f: vec![5],
        g: vec![6],
        h: vec![7],
    };

    let ret = sscanf!(CORRECT_INPUT, "Testing with {TestStruct}!").unwrap();
    assert_eq!(ret, correct);

    for input in WRONG_INPUTS {
        let res = sscanf!(input, "Testing with {TestStruct}!");
        res.unwrap_err();
    }
}

#[test]
fn mapper() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({d},{b},{c},{a})")]
    struct TestStruct {
        #[sscanf(map = |x: char| x.to_digit(10).unwrap() as u8)]
        a: u8,
        #[sscanf(map = |x: usize| x.to_string())]
        b: String,
        #[sscanf(map = |x: i8| x as isize)]
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
fn lifetimes() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({name},{age},{address})")]
    struct Person<'a, 'b> {
        name: &'a str,
        age: u8,
        address: &'b str,
        #[sscanf(default = "")]
        source: &'static str,
    }

    let input = String::from("Hi, I'm (Bob,42,here)!");
    let bob = sscanf!(input, "Hi, I'm {Person}!").unwrap();

    assert_eq!(bob.name, "Bob");
    assert_eq!(bob.age, 42);
    assert_eq!(bob.address, "here");
}

#[test]
fn lifetime_static() {
    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({name},{age},{address})")]
    struct Person {
        name: &'static str,
        age: u8,
        address: &'static str,
    }

    let input = "Hi, I'm (Bob,42,here)!";
    let bob = sscanf!(input, "Hi, I'm {Person}!").unwrap();

    assert_eq!(bob.name, "Bob");
    assert_eq!(bob.age, 42);
    assert_eq!(bob.address, "here");
}

#[test]
fn generics() {
    use std::str::FromStr;

    #[derive(FromScanf, Debug, PartialEq)]
    #[sscanf(format = "({name},{age},{data:/[a-z]+/})")]
    struct Person<T>
    where
        T: FromStr + 'static,
        <T as FromStr>::Err: std::error::Error + 'static,
    {
        name: String,
        age: u8,
        data: T,
    }

    let input = "Hi, I'm (Bob,42,here)!";
    let bob = sscanf!(input, "Hi, I'm {Person<String>}!").unwrap();

    assert_eq!(bob.name, "Bob");
    assert_eq!(bob.age, 42);
    assert_eq!(bob.data, "here");
}
