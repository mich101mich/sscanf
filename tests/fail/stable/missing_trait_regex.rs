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
