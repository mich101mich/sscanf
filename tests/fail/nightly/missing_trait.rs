mod module {
    pub struct NoFromScanf;
}

fn main() {
    sscanf::sscanf!("hi", "{}", module::NoFromScanf);
    sscanf::sscanf!("hi", "{module::NoFromScanf}");
    use module::NoFromScanf;
    sscanf::sscanf!("hi", "{}", NoFromScanf);
    sscanf::sscanf!("hi", "{NoFromScanf}");
}
