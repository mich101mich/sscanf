fn main() {
    ///// lib.rs: crate docs
    use sscanf::sscanf;
    sscanf!("", "Too many placeholders: {}{}{}", usize);
    //~                                     ^^ more placeholders than types provided
    //~                                   ^^ more placeholders than types provided

    ///// macros.rs: sscanf_with_regex!
    use sscanf::sscanf_with_regex;
    let input = "...";
    sscanf_with_regex!(input, r"a(b{usize}c)+d"); // won't work, because the regex is split into "a(b", "c)+d" and the placeholder {usize}
    //~                         ^^^ invalid regex syntax in literal part: regex parse error:
    //~                                 a(b
    //~                                  ^
    //~                             error: unclosed group

    ///// macros.rs: sscanf!
    // temporary value: does not work
    use sscanf::sscanf;
    sscanf!(String::from("5"), "{usize}");
    //~     ^^^^^^^^^^^^^^^^^ error: temporary value dropped while borrowed
    //~                       label: creates a temporary value which is freed while still in use

    ///// from_scanf.rs: FromScanf
    #[derive(sscanf::FromScanf)]
    #[sscanf("{first} {last}")]
    struct Name<'a, 'b> {
        first: &'a str,
        last: &'b str,
    }
    // ...same impl setup as above...
    let parsed;
    {
        let input = String::from("John Doe"); // locally owned string
        parsed = sscanf::sscanf!(input, "{Name}").unwrap();
        //~                      ^^^^^ error: `input` does not live long enough
        //~                            label: borrowed value does not live long enough
        // input is dropped here
    }
    println!("{} {}", parsed.first, parsed.last); // use after drop

    ////////////////////////////////////////////////////////////////////////////////
}
