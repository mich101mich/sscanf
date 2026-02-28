fn main() {
    ///// Raw string with missing type argument /////
    sscanf::sscanf!("hi", r"asdf{}asdf");
    //~                         ^^ more placeholders than types provided

    ///// Raw string with invalid number format /////
    sscanf::sscanf!("hi", r"asdf{:bob}asdf", usize);
    //~                            ^ multiple number format options are not allowed

    ///// Raw string with invalid format option in placeholder /////
    sscanf::sscanf!("hi", r"asdf{usize:bob}asdf");
    //~                                 ^ multiple number format options are not allowed

    ///// Raw string with hashes and missing type argument /////
    sscanf::sscanf!("hi", r##"asdf{}asdf"##);
    //~                           ^^ more placeholders than types provided

    ///// Raw string with hashes and invalid number format /////
    sscanf::sscanf!("hi", r##"asdf{:bob}asdf"##, usize);
    //~                              ^ multiple number format options are not allowed

    ///// Raw string with hashes and invalid format option in placeholder /////
    sscanf::sscanf!("hi", r##"asdf{usize:bob}asdf"##);
    //~                                   ^ multiple number format options are not allowed

    ////////////////////////////////////////////////////////////////////////////////
}
