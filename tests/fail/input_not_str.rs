fn main() {
    ///// Incorrect input type (usize) /////
    sscanf::sscanf!(5usize, "{usize}");
    //~             ^^^^^^ error: mismatched types
    //~                    label: expected `&str`, found `&usize`

    ///// Incorrect input type (bytes) /////
    sscanf::sscanf!(b"5", "{usize}");
    //~             ^^^^ error: mismatched types
    //~                  label: expected `&str`, found `&&[u8; 1]`

    ///// Temporary value (example from sscanf docs) /////
    sscanf::sscanf!(String::from("5"), "{usize}");
    //~             ^^^^^^^^^^^^^^^^^ error: temporary value dropped while borrowed
    //~                               label: creates a temporary value which is freed while still in use

    ////////////////////////////////////////////////////////////////////////////////
}
