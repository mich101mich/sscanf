fn main() {
    ///// Invalid regex flag /////
    sscanf::sscanf!("", "{://g}", str);
    //~                 ^^^^^^^^ unknown format option starting with 'g'.
    //~                          Use 'b', 'o', 'x' or 'r' for number format options, '/' for regex, or '[' for custom format options:
    //~                           --> stable/invalid_custom_regex.rs:2:30
    //~                            |
    //~                          2 | "{://g}"
    //~                            |      ^

    ///// Empty regex /////
    sscanf::sscanf!("", "{:/}", str);
    //~                 ^^^^^^ missing '/' to close the regex option:
    //~                         --> stable/invalid_custom_regex.rs:2:28
    //~                          |
    //~                        2 | "{:/}"
    //~                          |    ^^

    ///// Unclosed regex /////
    sscanf::sscanf!("", "{:/", str);
    //~                 ^^^^^ missing '/' to close the regex option:
    //~                        --> stable/invalid_custom_regex.rs:2:28
    //~                         |
    //~                       2 | "{:/"
    //~                         |    ^

    ///// Unclosed regex with escaped delimiter /////
    sscanf::sscanf!("", r"{:/\", str);
    //~                 ^^^^^^^ missing '/' to close the regex option:
    //~                          --> stable/invalid_custom_regex.rs:2:29
    //~                           |
    //~                         2 | r"{:/\"
    //~                           |     ^^

    ///// Unclosed regex with escaped backslash /////
    sscanf::sscanf!("", "{:/\\", str);
    //~                 ^^^^^^^ missing '/' to close the regex option:
    //~                          --> stable/invalid_custom_regex.rs:2:28
    //~                           |
    //~                         2 | "{:/\\"
    //~                           |    ^^^

    ///// Unclosed regex with escaped slash /////
    sscanf::sscanf!("", r"{:/\/", str);
    //~                 ^^^^^^^^ missing unescaped '/' to close the regex option:
    //~                           --> stable/invalid_custom_regex.rs:2:29
    //~                            |
    //~                          2 | r"{:/\/"
    //~                            |     ^^^

    ///// Unclosed regex with escaped slash (standard string) /////
    sscanf::sscanf!("", "{:/\\/", str);
    //~                 ^^^^^^^^ missing unescaped '/' to close the regex option:
    //~                           --> stable/invalid_custom_regex.rs:2:28
    //~                            |
    //~                          2 | "{:/\\/"
    //~                            |    ^^^^

    ///// Invalid regex (unclosed character class) /////
    sscanf::sscanf!("", r"{:/abc[def/}", str);
    //~                 ^^^^^^^^^^^^^^^ invalid regex override: regex parse error:
    //~                                     abc[def
    //~                                        ^
    //~                                 error: unclosed character class:
    //~                                  --> stable/invalid_custom_regex.rs:2:29
    //~                                   |
    //~                                 2 | r"{:/abc[def/}"
    //~                                   |     ^^^^^^^^^

    ///// Invalid regex (repetition operator missing expression) /////
    sscanf::sscanf!("", r"{:/{/}", str);
    //~                 ^^^^^^^^^ invalid regex override: regex parse error:
    //~                               {
    //~                               ^
    //~                           error: repetition operator missing expression:
    //~                            --> stable/invalid_custom_regex.rs:2:29
    //~                             |
    //~                           2 | r"{:/{/}"
    //~                             |     ^^^

    ///// Invalid regex (escaped brace repetition operator) /////
    sscanf::sscanf!("", r"{:/{\}/}", str);
    //~                 ^^^^^^^^^^^ invalid regex override: regex parse error:
    //~                                 {\}
    //~                                 ^
    //~                             error: repetition operator missing expression:
    //~                              --> stable/invalid_custom_regex.rs:2:29
    //~                               |
    //~                             2 | r"{:/{\}/}"
    //~                               |     ^^^^^

    ////////////////////////////////////////////////////////////////////////////////
}
