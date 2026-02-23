fn main() {
    ///// Unclosed placeholder { /////
    sscanf::sscanf!("", "{", str);
    //~                 ^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //~                      --> stable/invalid_placeholder.rs:2:26
    //~                       |
    //~                     2 | "{"
    //~                       |  ^

    ///// Unclosed placeholder {str /////
    sscanf::sscanf!("", "{str");
    //~                 ^^^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //~                         --> stable/invalid_placeholder.rs:2:26
    //~                          |
    //~                        2 | "{str"
    //~                          |  ^^^^

    ///// Unclosed placeholder {: /////
    sscanf::sscanf!("", "{:", str);
    //~                 ^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //~                       --> stable/invalid_placeholder.rs:2:26
    //~                        |
    //~                      2 | "{:"
    //~                        |  ^^

    ///// Unclosed placeholder {:b /////
    sscanf::sscanf!("", "{:b", str);
    //~                 ^^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //~                        --> stable/invalid_placeholder.rs:2:26
    //~                         |
    //~                       2 | "{:b"
    //~                         |  ^^^

    ///// Unclosed placeholder {:// /////
    sscanf::sscanf!("", "{://", str);
    //~                 ^^^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{':
    //~                         --> stable/invalid_placeholder.rs:2:26
    //~                          |
    //~                        2 | "{://"
    //~                          |  ^^^^

    ///// Placeholder with escaped closing brace /////
    sscanf::sscanf!("", "{:}}", str);
    //~                 ^^^^^^ escaped '}}' after an unescaped '{'.
    //~                        If you didn't mean to create a placeholder, escape the '{' as '{{'
    //~                        If you did, either remove the second '}' or escape it with another '}':
    //~                         --> stable/invalid_placeholder.rs:2:29
    //~                          |
    //~                        2 | "{:}}"
    //~                          |     ^

    ///// Unexpected closing brace /////
    sscanf::sscanf!("", "}", str);
    //~                 ^^^ unexpected standalone '}'. Literal '}' need to be escaped as '}}':
    //~                      --> stable/invalid_placeholder.rs:2:26
    //~                       |
    //~                     2 | "}"
    //~                       |  ^

    ///// Unexpected closing brace with colon /////
    sscanf::sscanf!("", ":}", str);
    //~                 ^^^^ unexpected standalone '}'. Literal '}' need to be escaped as '}}':
    //~                       --> stable/invalid_placeholder.rs:2:27
    //~                        |
    //~                      2 | ":}"
    //~                        |   ^

    ///// Escaped opening brace with single closing brace /////
    sscanf::sscanf!("", "{{:}", str);
    //~                 ^^^^^^ unexpected standalone '}'. Literal '}' need to be escaped as '}}':
    //~                         --> stable/invalid_placeholder.rs:2:29
    //~                          |
    //~                        2 | "{{:}"
    //~                          |     ^

    ///// Regex missing colon prefix /////
    sscanf::sscanf!("", "{/.*?/}", str);
    //~                 ^^^^^^^^^ missing `:` in front of custom regex. Write `{:/.*?/}` instead:
    //~                            --> stable/invalid_placeholder.rs:2:27
    //~                             |
    //~                           2 | "{/.*?/}"
    //~                             |   ^^^^^

    ////////////////////////////////////////////////////////////////////////////////
}
