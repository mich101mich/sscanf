fn main() {
    ///// Invalid hashtag position after number format /////
    sscanf::sscanf!("", "{:#x#}", u8);
    //~                 ^^^^^^^^ unexpected hashtag '#' after number format option:
    //~                           --> stable/invalid_radix_option.rs:2:30
    //~                            |
    //~                          2 | "{:#x#}"
    //~                            |      ^

    ///// Radix option with hashtag /////
    sscanf::sscanf!("", "{:#r16}", u8);
    //~                 ^^^^^^^^^ radix option 'r' cannot be used with a hashtag since it can't have a prefix:
    //~                            --> stable/invalid_radix_option.rs:2:28
    //~                             |
    //~                           2 | "{:#r16}"
    //~                             |    ^

    ///// Radix option with trailing hashtag /////
    sscanf::sscanf!("", "{:r16#}", u8);
    //~                 ^^^^^^^^^ unexpected hashtag '#' after number format option:
    //~                            --> stable/invalid_radix_option.rs:2:31
    //~                             |
    //~                           2 | "{:r16#}"
    //~                             |       ^

    ///// Radix option missing radix number /////
    sscanf::sscanf!("", "{:r}", u8);
    //~                 ^^^^^^ radix option 'r' has to be followed by a number:
    //~                         --> stable/invalid_radix_option.rs:2:28
    //~                          |
    //~                        2 | "{:r}"
    //~                          |    ^

    ///// Invalid radix number (too large) /////
    sscanf::sscanf!("", "{:r99}", u8);
    //~                 ^^^^^^^^ radix has to be a number between 2 and 36:
    //~                           --> stable/invalid_radix_option.rs:2:28
    //~                            |
    //~                          2 | "{:r99}"
    //~                            |    ^^^

    ///// Invalid radix number (too small) /////
    sscanf::sscanf!("", "{:r1}", u8);
    //~                 ^^^^^^^ radix has to be a number between 2 and 36:
    //~                          --> stable/invalid_radix_option.rs:2:28
    //~                           |
    //~                         2 | "{:r1}"
    //~                           |    ^^

    ////////////////////////////////////////////////////////////////////////////////
}
