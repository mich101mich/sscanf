fn main() {
    ///// Unclosed placeholder { /////
    sscanf::sscanf!("", "{", str);
    //~                  ^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'

    ///// Unclosed placeholder {str /////
    sscanf::sscanf!("", "{str");
    //~                  ^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'

    ///// Unclosed placeholder {: /////
    sscanf::sscanf!("", "{:", str);
    //~                  ^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'

    ///// Unclosed placeholder {:b /////
    sscanf::sscanf!("", "{:b", str);
    //~                  ^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'

    ///// Unclosed placeholder {:// /////
    sscanf::sscanf!("", "{://", str);
    //~                  ^^^^ missing '}' to close a placeholder. If the '{' was intended to be a literal, escape it with '{{'

    ///// Placeholder with escaped closing brace /////
    sscanf::sscanf!("", "{:}}", str);
    //~                     ^ escaped '}}' after an unescaped '{'.
    //~                       If you didn't mean to create a placeholder, escape the '{' as '{{'
    //~                       If you did, either remove the second '}' or escape it with another '}'

    ///// Unexpected closing brace /////
    sscanf::sscanf!("", "}", str);
    //~                  ^ unexpected standalone '}'. Literal '}' need to be escaped as '}}'

    ///// Unexpected closing brace with colon /////
    sscanf::sscanf!("", ":}", str);
    //~                   ^ unexpected standalone '}'. Literal '}' need to be escaped as '}}'

    ///// Escaped opening brace with single closing brace /////
    sscanf::sscanf!("", "{{:}", str);
    //~                     ^ unexpected standalone '}'. Literal '}' need to be escaped as '}}'

    ///// Regex missing colon prefix /////
    sscanf::sscanf!("", "{/.*?/}", str);
    //~                   ^^^^^ missing `:` in front of custom regex. Write `{:/.*?/}` instead

    ////////////////////////////////////////////////////////////////////////////////
}
