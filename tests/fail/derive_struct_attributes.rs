#[derive(sscanf::FromScanf)]
struct TestNoAttributes;

#[derive(sscanf::FromScanf)]
#[sscanf]
struct TestEmptyAttribute;

#[derive(sscanf::FromScanf)]
#[sscanf()]
struct TestEmptyAttribute2;

#[derive(sscanf::FromScanf)]
#[sscanf = ""]
struct TestAssignedAttribute;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "", format_unescaped = "")]
struct TestTooManyAttributes;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "", format_unescaped = "", transparent)]
struct TestTooManyAttributes2;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
#[sscanf(format = "")]
struct TestMultipleAttributes;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
#[sscanf(format_unescaped = "")]
struct TestMultipleDifferentAttributes;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "", bob = "")]
struct TestInvalidAttribute;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
#[sscanf(bob = "")]
struct TestMultipleInvalidAttribute;

#[derive(sscanf::FromScanf)]
#[sscanf(format = "" bob = "")]
struct TestMissingComma;

#[derive(sscanf::FromScanf)]
#[sscanf(format = 5)]
struct TestFormatNotString;

#[derive(sscanf::FromScanf)]
#[sscanf(format =)]
struct TestFormatMissingValue;

#[derive(sscanf::FromScanf)]
#[sscanf(format "")]
struct TestFormatMissingEquals;

#[derive(sscanf::FromScanf)]
#[sscanf(format)]
struct TestFormatJustIdent;

#[derive(sscanf::FromScanf)]
#[sscanf(= "")]
struct TestFormatMissingName;

#[derive(sscanf::FromScanf)]
#[sscanf(default)]
struct TestDefaultOnStruct;

#[derive(sscanf::FromScanf)]
#[sscanf(default = "")]
struct TestDefaultWithValueOnStruct;

#[derive(sscanf::FromScanf)]
#[sscanf(bob)]
struct TestInvalidIdent;

#[derive(sscanf::FromScanf)]
#[sscanf(formt)]
struct TestTypoInIdent;

#[derive(sscanf::FromScanf)]
#[sscanf(formad_unscabededd)]
struct TestMoreTyposInIdent;

#[derive(sscanf::FromScanf)]
#[sscanf(defauld)]
struct TestTyposAndWrongIdent;

#[derive(sscanf::FromScanf)]
#[sscanf(transparent)]
struct TestTransparentNoField;

#[derive(sscanf::FromScanf)]
#[sscanf(transparent)]
struct TestTransparentMultiField(usize, u8);

#[derive(sscanf::FromScanf)]
#[sscanf(transparent(5))]
struct TestTransparentArg(usize);

#[derive(sscanf::FromScanf)]
#[sscanf(transparent = "true")]
struct TestTransparentValue(usize);

fn main() {}
