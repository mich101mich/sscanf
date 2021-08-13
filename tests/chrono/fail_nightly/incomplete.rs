fn main() {
    sscanf::scanf!("hi", "{%.s}", DateTime);
    sscanf::scanf!("hi", "{%:b}", DateTime);
    sscanf::scanf!("hi", "{%:}", DateTime);
    sscanf::scanf!("hi", "{%.3a}", DateTime);
    sscanf::scanf!("hi", "{%38}", DateTime);
    sscanf::scanf!("hi", "{%}", DateTime);
    sscanf::scanf!("hi", "{%%%}", DateTime);
}
