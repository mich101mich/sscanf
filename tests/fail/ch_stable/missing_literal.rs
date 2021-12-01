fn main() {
    sscanf::scanf!("5");
    sscanf::scanf!("5",);
    
    let input = "hi";
    sscanf::scanf!(input, '{', std::vec::Vec<usize>);

    let fmt = "{}";
    sscanf::scanf!(input, fmt, std::vec::Vec<usize>);
}
