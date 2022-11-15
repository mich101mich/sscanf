fn main() {
    sscanf::sscanf!("5");
    sscanf::sscanf!("5",);
    
    let input = "hi";
    sscanf::sscanf!(input, '{', std::vec::Vec<usize>);

    let fmt = "{}";
    sscanf::sscanf!(input, fmt, std::vec::Vec<usize>);
}
