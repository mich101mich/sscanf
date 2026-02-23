fn main() {
    ///// Call with no arguments /////
    sscanf::sscanf!();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: sscanf: at least 2 Parameters required: Input and format string
    //  --> missing_params.rs:2:5
    //   |
    // 2 |     sscanf::sscanf!();
    //   |     ^^^^^^^^^^^^^^^^^
    //   |
    //   = note: this error originates in the macro `sscanf::sscanf` (in Nightly builds, run with -Z macro-backtrace for more info)

    ///// Call with only input string /////
    sscanf::sscanf!("5");
    //~                ^ sscanf: at least 2 Parameters required: Missing format string

    ///// Call with input string and trailing comma /////
    sscanf::sscanf!("5",);
    //~                 ^ at least 2 Parameters required: Missing format string

    ///// Invalid format string type (char) /////
    sscanf::sscanf!("5", 'x');
    //~                  ^^^ expected string literal

    ///// Invalid format string type (type) /////
    sscanf::sscanf!("5", usize);
    //~                  ^^^^^ expected string literal

    ////////////////////////////////////////////////////////////////////////////////
}
