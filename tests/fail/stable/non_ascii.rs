fn main() {
    type Ay̆y̆y̆ = u8;

    ///// Non-ASCII identifier with invalid format option /////
    sscanf::sscanf!("Hi", "y̆😛y̆{Ay̆y̆y̆:😛}y̆😛y̆");
    //~                   ^^^^^^^^^^^^^^^^^^^ unknown format option starting with '😛'.
    //~                                       Use 'b', 'o', 'x' or 'r' for number format options, '/' for regex, or '[' for custom format options:
    //~                                        --> stable/non_ascii.rs:4:38
    //~                                         |
    //~                                       4 | "y̆😛y̆{Ay̆y̆y̆:😛}y̆😛y̆"
    //~                                         |            ^^

    ///// Non-ASCII identifier in raw string with invalid format option /////
    sscanf::sscanf!("Hi", r##"y̆👨‍👩‍👧‍👦y̆{Ay̆y̆y̆:😛}y̆😛y̆"##);
    //~                   ^^^^^^^^^^^^^^^^^^^^^^^^ unknown format option starting with '😛'.
    //~                                            Use 'b', 'o', 'x' or 'r' for number format options, '/' for regex, or '[' for custom format options:
    //~                                             --> stable/non_ascii.rs:4:50
    //~                                              |
    //~                                            4 | r##"y̆👨👩👧👦y̆{Ay̆y̆y̆:😛}y̆😛y̆"##
    //~                                              |                        ^^

    ///// Unicode escape sequences with invalid format option /////
    #[rustfmt::skip]
    sscanf::sscanf!("Hi", "y\u{306}\u{1f61b}😛y\u{306}{Ay\u{306}y\u{306}y\u{306}:\u{1f61b}}y\u{306}\u{1f61b}y\u{306}");
    //~                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unknown format option starting with '😛'.
    //~                                                                                                               Use 'b', 'o', 'x' or 'r' for number format options, '/' for regex, or '[' for custom format options:
    //~                                                                                                                --> stable/non_ascii.rs:5:82
    //~                                                                                                                 |
    //~                                                                                                               5 | "y\u{306}\u{1f61b}😛y\u{306}{Ay\u{306}y\u{306}y\u{306}:\u{1f61b}}y\u{306}\u{1f61b}y\u{306}"
    //~                                                                                                                 |                                                        ^^^^^^^^^

    ////////////////////////////////////////////////////////////////////////////////
}
