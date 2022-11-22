fn main() {
    sscanf::sscanf!("hi", "asdf{x}asdf", usize);
    sscanf::sscanf!("hi", "asdf{b}asdf", usize);
    sscanf::sscanf!("hi", "asdf{o}asdf", usize);
    sscanf::sscanf!("hi", "asdf{r4}asdf", usize);
    sscanf::sscanf!("hi", "asdf{r13}asdf", usize);
    sscanf::sscanf!("hi", "asdf{/.*/}asdf", usize);
}