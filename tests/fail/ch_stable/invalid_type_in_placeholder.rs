struct NoRegex;
impl std::str::FromStr for NoRegex {
    type Err = std::convert::Infallible;
    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(NoRegex)
    }
}
mod module {
    pub struct NoFromStr;
    impl sscanf::RegexRepresentation for NoFromStr {
        const REGEX: &'static str = ".*";
    }
}

fn main() {
    // path in placeholder
    sscanf::scanf!("hi", "{std::vec::Vec<usize>}");
    sscanf::scanf!("hi", "{module::NoFromStr}");

    // missing trait
    sscanf::scanf!("hi", "{NoRegex}");

    // invalid type
    sscanf::scanf!("hi", "{3}");
    sscanf::scanf!("hi", "{.}");
    sscanf::scanf!("hi", "{bob}");
    sscanf::scanf!("hi", "{3:/hi/}");
    sscanf::scanf!("hi", "{.:/hi/}");
    sscanf::scanf!("hi", "{bob:/hi/}");
}
