fn main() {
    ///// Invalid syntax in placeholder {.} /////
    sscanf::sscanf!("hi", "{.}");
    //~                   ^^^^^ invalid type in placeholder: expected identifier.
    //~                         Hint: The syntax for placeholders is {<type>} or {<type>:<config>}. Make sure <type> is a valid type or index.
    //~                         If you want syntax highlighting and better errors, place the type in the arguments after the format string while debugging:
    //~                          --> stable/invalid_type_in_placeholder.rs:2:29
    //~                           |
    //~                         2 | "{.}"
    //~                           |   ^

    ///// Non-existent type in placeholder {bob} /////
    sscanf::sscanf!("hi", "{bob}");
    //~                   ^^^^^^^ error: cannot find type `bob` in this scope
    //~                           label: not found in this scope

    ///// Invalid syntax with regex in placeholder {.:/hi/} /////
    sscanf::sscanf!("hi", "{.:/hi/}");
    //~                   ^^^^^^^^^^ invalid type in placeholder: expected identifier.
    //~                              Hint: The syntax for placeholders is {<type>} or {<type>:<config>}. Make sure <type> is a valid type or index.
    //~                              If you want syntax highlighting and better errors, place the type in the arguments after the format string while debugging:
    //~                               --> stable/invalid_type_in_placeholder.rs:2:29
    //~                                |
    //~                              2 | "{.:/hi/}"
    //~                                |   ^

    ///// Non-existent type with regex in placeholder {bob:/hi/} /////
    sscanf::sscanf!("hi", "{bob:/hi/}");
    //~                   ^^^^^^^^^^^^ error: cannot find type `bob` in this scope
    //~                                label: not found in this scope

    ///// Placeholder index out of range /////
    sscanf::sscanf!("hi", "{99}");
    //~                   ^^^^^^ type index 99 out of range of 0 types:
    //~                           --> stable/invalid_type_in_placeholder.rs:2:29
    //~                            |
    //~                          2 | "{99}"
    //~                            |   ^^

    ////////////////////////////////////////////////////////////////////////////////
}
