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

    // error: missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //          --> stable/multiline_format_str.rs:4:10
    //           |
    //         4 |   "{This
    //           |  __^
    //         5 | | is
    //         6 | | a
    //        ...  |
    //        12 | | really
    //        13 | | long!"
    //           | |_____^
    //
    //   --> stable/multiline_format_str.rs:4:9
    //    |
    //  4 | /         "{This
    //  5 | | is
    //  6 | | a
    //  7 | | faulty
    // ...  |
    // 12 | | really
    // 13 | | long!"
    //    | |______^

    ///// Multiline string with unclosed placeholder in middle /////
    sscanf::sscanf!(
        "Hi",
        "This
is another fau{ty
multiline string!"
    )
    .unwrap();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //         --> stable/multiline_format_str.rs:5:15
    //          |
    //        5 |   is another fau{ty
    //          |  _______________^
    //        6 | | multiline string!"
    //          | |_________________^
    //
    //  --> stable/multiline_format_str.rs:4:9
    //   |
    // 4 | /         "This
    // 5 | | is another fau{ty
    // 6 | | multiline string!"
    //   | |__________________^

    ///// Multiline string with unexpected closing brace /////
    sscanf::sscanf!(
        "Hi",
        "And so
is
this}, but
the error is only
on one line"
    )
    .unwrap();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: unexpected standalone '}'. Literal '}' need to be escaped as '}}':
    //         --> stable/multiline_format_str.rs:6:5
    //          |
    //        6 | this}, but
    //          |     ^
    //
    //  --> stable/multiline_format_str.rs:4:9
    //   |
    // 4 | /         "And so
    // 5 | | is
    // 6 | | this}, but
    // 7 | | the error is only
    // 8 | | on one line"
    //   | |____________^

    ///// Multiline string with unicode and unclosed placeholder /////
    sscanf::sscanf!(
        "Hi",
        "This
is a 😟 fau{ty
multiline string with unicode!"
    )
    .unwrap();

    //~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

    // error: missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //         --> stable/multiline_format_str.rs:5:12
    //          |
    //        5 |   is a 😟 fau{ty
    //          |  ____________^
    //        6 | | multiline string with unicode!"
    //          | |______________________________^
    //
    //  --> stable/multiline_format_str.rs:4:9
    //   |
    // 4 | /         "This
    // 5 | | is a 😟 fau{ty
    // 6 | | multiline string with unicode!"
    //   | |_______________________________^

    ///// Escaped newlines with unclosed placeholder /////
    sscanf::sscanf!("Hi", "This\nis a\n{fake\nmultiline string!").unwrap();
    //~                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //~                                                           --> stable/multiline_format_str.rs:2:40
    //~                                                            |
    //~                                                          2 | "This\nis a\n{fake\nmultiline string!"
    //~                                                            |              ^^^^^^^^^^^^^^^^^^^^^^^^

    ////////////////////////////////////////////////////////////////////////////////
}
