fn main() {
    ///// Missing type for placeholder /////
    sscanf::sscanf!("hi", "asdf{}{usize}as{}df{i32}", usize);
    //~                                   ^^ more placeholders than types provided

    ///// Missing type for multiple placeholders /////
    sscanf::sscanf!("hi", "asdf{}{}asdf{}", usize);
    //~                                ^^ more placeholders than types provided
    //~                          ^^ more placeholders than types provided

    ////////////////////////////////////////////////////////////////////////////////
}
