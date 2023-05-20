#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoValue(#[sscanf(default =)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoEquals(#[sscanf(default 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoIdent(#[sscanf(= 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestFormatInField(#[sscanf(format = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestImpliedFormatInField(#[sscanf("")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestUnknownArg(#[sscanf(bob = 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestTypoInArg(#[sscanf(mao)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestMoreTyposInArg(#[sscanf(defold)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDuplicateArg(#[sscanf(default = 5, default = 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDuplicateMultiArg(#[sscanf(default = 5)] #[sscanf(default = 5)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultAndMap(#[sscanf(default = 5, map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultAndMapMulti(#[sscanf(default = 5)] #[sscanf(map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoPlaceholder(#[sscanf(map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoAssign(#[sscanf(map)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoValue(#[sscanf(map =)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNotClosure(#[sscanf(map = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoParam(#[sscanf(map = || { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoType(#[sscanf(map = |x| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapMoreTypes(#[sscanf(map = |x: usize, y: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFilterMapNoOption(#[sscanf(filter_map = |x: usize| { x as u8 })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFilterMapNoValue(#[sscanf(filter_map)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFromNoType(#[sscanf(from)] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFromNotType(#[sscanf(from = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestTryFromNoType(#[sscanf(try_from)] u8);


// mismatched types appear at the end of the error message

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultWrongType(#[sscanf(default = "")] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapWrongReturn(#[sscanf(map = |x: usize| { x })] u8);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultNoDefault(#[sscanf(default)] std::num::ParseIntError);

#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFromNoFrom(#[sscanf(from = f32)] u8);


fn main() {}
