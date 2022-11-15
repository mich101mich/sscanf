mod module {
    pub struct NoFromStr;
    impl sscanf::RegexRepresentation for NoFromStr {
        const REGEX: &'static str = ".*";
    }
}

fn main() {
    sscanf::sscanf!("hi", "{}", module::NoFromStr);
    sscanf::sscanf!("hi", "{module::NoFromStr}");
    use module::NoFromStr;
    sscanf::sscanf!("hi", "{}", NoFromStr);
    sscanf::sscanf!("hi", "{NoFromStr}");
}
