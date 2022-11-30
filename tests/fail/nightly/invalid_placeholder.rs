fn main() {
    sscanf::sscanf!("", "{", str);
    sscanf::sscanf!("", "{str");
    sscanf::sscanf!("", "{:", str);
    sscanf::sscanf!("", "{:b", str);
    sscanf::sscanf!("", "{://", str);
    sscanf::sscanf!("", "{:}}", str);
    sscanf::sscanf!("", "}", str);
    sscanf::sscanf!("", ":}", str);
    sscanf::sscanf!("", "{{:}", str);
    sscanf::sscanf!("", "{/.*?/}", str);
}
