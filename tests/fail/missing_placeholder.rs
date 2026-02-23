fn main() {
    ///// More arguments than placeholders /////
    #[rustfmt::skip]
    sscanf::sscanf!("hi", "asdf{}", usize, std::vec::Vec<std::string::String>, u32);
    //~                                                                        ^^^ unused type
    //~                                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unused type

    ///// Unused type argument with typed placeholders /////
    sscanf::sscanf!("hi", "asdf{u32}{}{usize}", usize, u32);
    //~                                                ^^^ unused type

    ////////////////////////////////////////////////////////////////////////////////
}
