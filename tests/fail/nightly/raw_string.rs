fn main() {
    sscanf::scanf!("hi", r"asdf{}asdf");
    sscanf::scanf!("hi", r"asdf{:bob}asdf", usize);
    sscanf::scanf!("hi", r"asdf{usize:bob}asdf");
    sscanf::scanf!("hi", r##"asdf{}asdf"##);
    sscanf::scanf!("hi", r##"asdf{:bob}asdf"##, usize);
    sscanf::scanf!("hi", r##"asdf{usize:bob}asdf"##);
}
