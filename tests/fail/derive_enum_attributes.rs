#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
enum TestOuterFormat { A, B }

#[derive(sscanf::FromScanf)]
enum TestNoFormat { A, B }

fn main() {}
