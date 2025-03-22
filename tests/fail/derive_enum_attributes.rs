#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
enum TestOuterFormat { A, B }

#[derive(sscanf::FromScanf)]
enum TestNoFormat { A, B }

#[derive(sscanf::FromScanf)]
enum TestNoVariants { }

#[derive(sscanf::FromScanf)]
#[sscanf(from_name)]
enum TestAutogenHasFields { A, B(usize) }

#[derive(sscanf::FromScanf)]
#[sscanf(from_name)]
enum TestAutogenAllSkip { #[sscanf(skip)] A, #[sscanf(skip)] B }

#[derive(sscanf::FromScanf)]
#[sscanf(from_name = {})]
enum TestAutogenInvalid { A, B }

#[derive(sscanf::FromScanf)]
#[sscanf(from_name = "bob")]
enum TestAutogenInvalidWord { A, B }

#[derive(sscanf::FromScanf)]
#[sscanf(from_name = "casesensitive")]
enum TestAutogenInvalidCase { A, B }


fn main() {}
