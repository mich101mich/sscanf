fn main() {
    sscanf::scanf!("hi", "{:%.s}", DateTime);
    sscanf::scanf!("hi", "{:%:b}", DateTime);
    sscanf::scanf!("hi", "{:%:}", DateTime);
    sscanf::scanf!("hi", "{DateTime:%.3a}");
    sscanf::scanf!("hi", "{DateTime:%38}");
    sscanf::scanf!("hi", "{DateTime:%}");
    sscanf::scanf!("hi", "{DateTime:%%%}");
}
