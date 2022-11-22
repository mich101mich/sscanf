fn main() {
    sscanf::sscanf!("hi", "asdf{}", usize, std::vec::Vec<std::string::String>, u32);
    sscanf::sscanf!("hi", "asdf{u32}{}{usize}", usize, u32);
}
