fn main() {
    let s;
    {
        let input = String::from("hi");
        s = sscanf::sscanf!(input, "{str}").unwrap();
    }
    println!("{}", s);
}