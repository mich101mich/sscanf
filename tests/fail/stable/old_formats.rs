fn main() {
    sscanf::scanf!("hi", "asdf{x}asdf", usize);
    sscanf::scanf!("hi", "asdf{b}asdf", usize);
    sscanf::scanf!("hi", "asdf{o}asdf", usize);
    sscanf::scanf!("hi", "asdf{r4}asdf", usize);
    sscanf::scanf!("hi", "asdf{r13}asdf", usize);
    sscanf::scanf!("hi", "asdf{/.*/}asdf", usize);
}