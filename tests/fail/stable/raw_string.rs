fn main() {
    ///// Raw string with missing type argument /////
    sscanf::sscanf!("hi", r"asdf{}asdf");
    //~                   ^^^^^^^^^^^^^ more placeholders than types provided:
    //~                                  --> stable/raw_string.rs:2:33
    //~                                   |
    //~                                 2 | r"asdf{}asdf"
    //~                                   |       ^^

    ///// Raw string with invalid number format /////
    sscanf::sscanf!("hi", r"asdf{:bob}asdf", usize);
    //~                   ^^^^^^^^^^^^^^^^^ multiple number format options are not allowed:
    //~                                      --> stable/raw_string.rs:2:36
    //~                                       |
    //~                                     2 | r"asdf{:bob}asdf"
    //~                                       |          ^

    ///// Raw string with invalid format option in placeholder /////
    sscanf::sscanf!("hi", r"asdf{usize:bob}asdf");
    //~                   ^^^^^^^^^^^^^^^^^^^^^^ multiple number format options are not allowed:
    //~                                           --> stable/raw_string.rs:2:41
    //~                                            |
    //~                                          2 | r"asdf{usize:bob}asdf"
    //~                                            |               ^

    ///// Raw string with hashes and missing type argument /////
    sscanf::sscanf!("hi", r##"asdf{}asdf"##);
    //~                   ^^^^^^^^^^^^^^^^^ more placeholders than types provided:
    //~                                      --> stable/raw_string.rs:2:35
    //~                                       |
    //~                                     2 | r##"asdf{}asdf"##
    //~                                       |         ^^

    ///// Raw string with hashes and invalid number format /////
    sscanf::sscanf!("hi", r##"asdf{:bob}asdf"##, usize);
    //~                   ^^^^^^^^^^^^^^^^^^^^^ multiple number format options are not allowed:
    //~                                          --> stable/raw_string.rs:2:38
    //~                                           |
    //~                                         2 | r##"asdf{:bob}asdf"##
    //~                                           |            ^

    ///// Raw string with hashes and invalid format option in placeholder /////
    sscanf::sscanf!("hi", r##"asdf{usize:bob}asdf"##);
    //~                   ^^^^^^^^^^^^^^^^^^^^^^^^^^ multiple number format options are not allowed:
    //~                                               --> stable/raw_string.rs:2:43
    //~                                                |
    //~                                              2 | r##"asdf{usize:bob}asdf"##
    //~                                                |                 ^

    ////////////////////////////////////////////////////////////////////////////////
}
