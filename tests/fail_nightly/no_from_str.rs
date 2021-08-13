mod bob {
    pub struct Test;
    impl sscanf::RegexRepresentation for Test {
        const REGEX: &'static str = "a";
    }
}
fn main() {
    sscanf::scanf!("hi", "{}", bob::Test);
}
