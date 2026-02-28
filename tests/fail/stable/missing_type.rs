fn main() {
    ///// Missing type for placeholder /////
    sscanf::sscanf!("hi", "asdf{}{usize}as{}df{i32}", usize);
    //~                   ^^^^^^^^^^^^^^^^^^^^^^^^^^ more placeholders than types provided:
    //~                                               --> stable/missing_type.rs:2:43
    //~                                                |
    //~                                              2 | "asdf{}{usize}as{}df{i32}"
    //~                                                |                 ^^

    ///// Missing type for multiple placeholders /////
    sscanf::sscanf!("hi", "asdf{}{}asdf{}", usize);
    //~                   ^^^^^^^^^^^^^^^^ more placeholders than types provided:
    //~                                     --> stable/missing_type.rs:2:34
    //~                                      |
    //~                                    2 | "asdf{}{}asdf{}"
    //~                                      |        ^^
    //~                   ^^^^^^^^^^^^^^^^ more placeholders than types provided:
    //~                                     --> stable/missing_type.rs:2:40
    //~                                      |
    //~                                    2 | "asdf{}{}asdf{}"
    //~                                      |              ^^

    ////////////////////////////////////////////////////////////////////////////////
}
