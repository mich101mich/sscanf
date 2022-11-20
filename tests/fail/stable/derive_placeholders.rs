#[derive(sscanf::FromScanf)]
#[sscanf(format = "{5} {x} {} {} {b} {} {} {b}")]
struct Test1 {
    a: u8,
    b: u8,
}

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{b}")]
struct Test2 {
    a: u8,
    #[sscanf(default = 5)]
    b: u8,
}

fn main() {}
