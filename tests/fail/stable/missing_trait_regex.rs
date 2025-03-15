mod module {
    pub struct NoRegex(pub u8);
    impl std::str::FromStr for NoRegex {
        type Err = std::convert::Infallible;
        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            Ok(NoRegex(0))
        }
    }
}

fn main() {
    sscanf::sscanf!("hi", "{}", module::NoRegex);
    sscanf::sscanf!("hi", "{module::NoRegex}");
    use module::NoRegex;
    sscanf::sscanf!("hi", "{}", NoRegex);
    sscanf::sscanf!("hi", "{NoRegex}");
}

// should be in tests/fail/derive_field_attributes.rs, but causes other error
// messages to go missing

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapParamNoScanf(#[sscanf(map = |x: module::NoRegex| { x.0 })] u8);
