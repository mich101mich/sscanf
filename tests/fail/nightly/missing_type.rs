fn main() {
    sscanf::sscanf!("hi", "asdf{}{}asdf{}", usize);
    sscanf::sscanf!("hi", "asdf{}{usize}as{}df{i32}", usize);
}
