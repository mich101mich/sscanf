fn main() {
    let s;
    {
        let input = String::from("hi");
        s = sscanf::scanf!(input, "{str}").unwrap();
    }
    println!("{}", s);
}