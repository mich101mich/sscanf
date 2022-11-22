struct NoRegex;
impl std::str::FromStr for NoRegex {
    type Err = std::convert::Infallible;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(NoRegex)
    }
}

fn main() {
    sscanf::sscanf!("hi", "{}", std::vec::Vec<usize>);
    sscanf::sscanf!("hi", "{}", NoRegex);
}

// should be in tests/fail/derive_field_attributes.rs, but causes other error
// messages to go missing

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapParamNoScanf(#[sscanf(map = |x: &[u8]| { x[0] })] u8);
