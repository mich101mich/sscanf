#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test1(#[sscanf(default =)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test2(#[sscanf(default 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test3(#[sscanf(= 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test4(#[sscanf(default = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test5(#[sscanf(format = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test6(#[sscanf(bob = 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test7(#[sscanf(default = 5, default = 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test8(#[sscanf(default = 5)] #[sscanf(default = 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test9(#[sscanf(default = 5, map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test10(#[sscanf(default = 5)] #[sscanf(map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct Test11(#[sscanf(map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct Test12(#[sscanf(map = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct Test13(#[sscanf(map)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct Test14(#[sscanf(map = || { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct Test15(#[sscanf(map = |x| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct Test16(#[sscanf(map = |x: usize, y: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct Test17(#[sscanf(map = |x: usize| { x })] u8);

fn main() {}

// expected expression after `=`

// duplicate attribute arg:
// duplicate attribute arg:

// expected closure expression for `map`

// expected closure expression for `map`

// expected `map` closure to take exactly one argument

// `map` closure has to specify the type of the argument

// unknown attribute arg

// cannot use both `default` and `map` on the same field

// attribute arg `{}` can only be used on the struct itself
