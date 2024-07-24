fn main() {
    let s;
    {
        let input = String::from("hi");
        s = sscanf::sscanf!(input, "{str}").unwrap();
    }
    println!("{}", s);

    #[derive(sscanf::FromScanf)]
    #[sscanf(format = "{}")]
    struct Wrapper<'a>(&'a str);

    let w;
    {
        let input = String::from("hi");
        w = sscanf::sscanf!(input, "{Wrapper}").unwrap();
    }
    println!("{}", w.0);
}
