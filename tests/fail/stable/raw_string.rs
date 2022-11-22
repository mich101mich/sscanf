fn main() {
    sscanf::sscanf!("hi", r"asdf{}asdf");
    sscanf::sscanf!("hi", r"asdf{:bob}asdf", usize);
    sscanf::sscanf!("hi", r"asdf{usize:bob}asdf");
    sscanf::sscanf!("hi", r##"asdf{}asdf"##);
    sscanf::sscanf!("hi", r##"asdf{:bob}asdf"##, usize);
    sscanf::sscanf!("hi", r##"asdf{usize:bob}asdf"##);
}
