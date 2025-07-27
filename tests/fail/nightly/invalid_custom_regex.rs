fn main() {
    sscanf::sscanf!("", "{://g}", str);
    sscanf::sscanf!("", "{:/}", str);
    sscanf::sscanf!("", "{:/", str);

    sscanf::sscanf!("", r"{:/\", str);
    sscanf::sscanf!("", "{:/\\", str);
    sscanf::sscanf!("", r"{:/\/", str);
    sscanf::sscanf!("", "{:/\\/", str);
}
