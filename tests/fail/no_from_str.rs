mod bob {
    pub struct Test;
    impl sscanf::RegexRepresentation for Test {
        fn regex() -> &'static str {
            "a"
        }
    }
}
fn main() {
    sscanf::scanf!("hi", "{}", bob::Test);
}
