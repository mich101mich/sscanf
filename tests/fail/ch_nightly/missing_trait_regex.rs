struct NoRegex;
impl std::str::FromStr for NoRegex {
    type Err = std::convert::Infallible;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(NoRegex)
    }
}

fn main() {
    sscanf::scanf!("hi", "{}", std::vec::Vec<usize>);
    sscanf::scanf!("hi", "{}", NoRegex);
}
