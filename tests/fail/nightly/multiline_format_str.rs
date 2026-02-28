fn main() {
    ///// Multiline string with unclosed placeholder at start /////
    sscanf::sscanf!(
        "Hi",
        "{This
is
a
faulty
multiline
string
that
is
really
long!"
    )
    .unwrap();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'
    //   --> nightly/multiline_format_str.rs:4:10
    //    |
    //  4 |           "{This
    //    |  __________^
    //  5 | | is
    //  6 | | a
    //  7 | | faulty
    // ...  |
    // 12 | | really
    // 13 | | long!"
    //    | |_____^

    ///// Multiline string with unclosed placeholder in middle /////
    sscanf::sscanf!(
        "Hi",
        "This
is another fau{ty
multiline string!"
    )
    .unwrap();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'
    //  --> nightly/multiline_format_str.rs:5:15
    //   |
    // 5 |   is another fau{ty
    //   |  _______________^
    // 6 | | multiline string!"
    //   | |_________________^

    ///// Multiline string with unexpected closing brace /////
    sscanf::sscanf!(
        "Hi",
        "And so
is
this}, but
//~ ^ unexpected standalone '}'. Literal '}' need to be escaped as '}}'
the error is only
on one line"
    )
    .unwrap();

    ///// Multiline string with unicode and unclosed placeholder /////
    sscanf::sscanf!(
        "Hi",
        "This
is a 😟 fau{ty
multiline string with unicode!"
    )
    .unwrap();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'
    //  --> nightly/multiline_format_str.rs:5:11
    //   |
    // 5 |   is a 😟 fau{ty
    //   |  ____________^
    // 6 | | multiline string with unicode!"
    //   | |______________________________^

    ///// Escaped newlines with unclosed placeholder /////
    sscanf::sscanf!("Hi", "This\nis a\n{fake\nmultiline string!").unwrap();
    //~                                ^^^^^^^^^^^^^^^^^^^^^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'

    ////////////////////////////////////////////////////////////////////////////////
}
