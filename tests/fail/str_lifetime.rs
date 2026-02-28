fn main() {
    ///// Borrowed str outliving input string /////
    let s;
    {
        let input = String::from("hi");
        s = sscanf::sscanf!(input, "{str}").unwrap();
        //~                 ^^^^^ error: `input` does not live long enough
        //~                       label: borrowed value does not live long enough
    }
    println!("{}", s);

    ///// Borrowed wrapper outliving input string /////
    #[derive(sscanf::FromScanf)]
    #[sscanf(format = "{}")]
    struct Wrapper<'a>(&'a str);

    let w;
    {
        let input = String::from("hi");
        w = sscanf::sscanf!(input, "{Wrapper}").unwrap();
        //~                 ^^^^^ error: `input` does not live long enough
        //~                       label: borrowed value does not live long enough
    }
    println!("{}", w.0);

    ////////////////////////////////////////////////////////////////////////////////
}
