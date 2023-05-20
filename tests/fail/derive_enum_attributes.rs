#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
enum TestOuterFormat { A, B }

#[derive(sscanf::FromScanf)]
enum TestNoFormat { A, B }

#[derive(sscanf::FromScanf)]
enum TestNoVariants { }

#[derive(sscanf::FromScanf)]
#[sscanf(autogen)]
enum TestAutogenHasFields { A, B(usize) }

#[derive(sscanf::FromScanf)]
#[sscanf(autogen)]
enum TestAutogenAllSkip { #[sscanf(skip)] A, #[sscanf(skip)] B }

#[derive(sscanf::FromScanf)]
#[sscanf(autogen = {})]
enum TestAutogenInvalid { A, B }

#[derive(sscanf::FromScanf)]
#[sscanf(autogen = "bob")]
enum TestAutogenInvalidWord { A, B }

#[derive(sscanf::FromScanf)]
#[sscanf(autogen = "casesensitive")]
enum TestAutogenInvalidCase { A, B }


fn main() {}
