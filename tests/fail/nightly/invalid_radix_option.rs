fn main() {
    ///// Invalid hashtag position after number format /////
    sscanf::sscanf!("", "{:#x#}", u8);
    //~                      ^ unexpected hashtag '#' after number format option

    ///// Radix option with hashtag /////
    sscanf::sscanf!("", "{:#r16}", u8);
    //~                    ^ radix option 'r' cannot be used with a hashtag since it can't have a prefix

    ///// Radix option with trailing hashtag /////
    sscanf::sscanf!("", "{:r16#}", u8);
    //~                       ^ unexpected hashtag '#' after number format option

    ///// Radix option missing radix number /////
    sscanf::sscanf!("", "{:r}", u8);
    //~                    ^ radix option 'r' has to be followed by a number

    ///// Invalid radix number (too large) /////
    sscanf::sscanf!("", "{:r99}", u8);
    //~                    ^^^ radix has to be a number between 2 and 36

    ///// Invalid radix number (too small) /////
    sscanf::sscanf!("", "{:r1}", u8);
    //~                    ^^ radix has to be a number between 2 and 36

    ////////////////////////////////////////////////////////////////////////////////
}
