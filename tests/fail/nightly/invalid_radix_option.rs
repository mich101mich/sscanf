fn main() {
    sscanf::sscanf!("", "{:#x#}", u8);
    sscanf::sscanf!("", "{:#r16}", u8);
    sscanf::sscanf!("", "{:r16#}", u8);
    sscanf::sscanf!("", "{:r}", u8);
    sscanf::sscanf!("", "{:r99}", u8);
    sscanf::sscanf!("", "{:r1}", u8);
    sscanf::sscanf!("", "{:x}", std::u8);
}
