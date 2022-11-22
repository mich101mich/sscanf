#[derive(sscanf::FromScanf)]
struct Test1;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "", format_unescaped = "")]
struct Test3;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "", bob = "")]
struct Test4;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "" bob = "")]
struct Test5;

#[derive(sscanf::FromScanf)]
#[sscanf(format = 5)]
struct Test6;

#[derive(sscanf::FromScanf)]
#[sscanf(format =)]
struct Test7;

#[derive(sscanf::FromScanf)]
#[sscanf(format "")]
struct Test8;

#[derive(sscanf::FromScanf)]
#[sscanf(= "")]
struct Test9;

#[derive(sscanf::FromScanf)]
#[sscanf(default)]
struct Test11;

#[derive(sscanf::FromScanf)]
#[sscanf(default = "")]
struct Test10;

#[derive(sscanf::FromScanf)]
#[sscanf(map = |x: usize| { x })]
struct Test12;

fn main() {}
