mod module {
    pub struct NoFromStr;
    impl sscanf::RegexRepresentation for NoFromStr {
        const REGEX: &'static str = ".*";
    }
}

fn main() {
    sscanf::scanf!("hi", "{}", module::NoFromStr);
    use module::NoFromStr;
    sscanf::scanf!("hi", "{}", NoFromStr);
}
