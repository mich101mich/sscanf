///// NoAttributes /////
#[derive(sscanf::FromScanf)]
struct TestNoAttributes;
//~    ^^^^^^^^^^^^^^^^ FromScanf: structs must have a format string as an attribute.
//~                     Please add either of #[sscanf(format = "...")], #[sscanf(format_regex = "...")] or #[sscanf("...")]

///// EmptyAttribute /////
#[derive(sscanf::FromScanf)]
#[sscanf]
struct TestEmptyAttribute;

//~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

// error: expected attribute arguments in parentheses: `sscanf(...)`
//  --> derive_struct_attributes.rs:2:3
//   |
// 2 | #[sscanf]
//   |   ^^^^^^

///// EmptyAttribute2 /////
#[derive(sscanf::FromScanf)]
#[sscanf()]
struct TestEmptyAttribute2;
//~    ^^^^^^^^^^^^^^^^^^^ FromScanf: structs must have a format string as an attribute.
//~                        Please add either of #[sscanf(format = "...")], #[sscanf(format_regex = "...")] or #[sscanf("...")]

///// AssignedAttribute /////
#[derive(sscanf::FromScanf)]
#[sscanf = ""]
struct TestAssignedAttribute;

//~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

// error: attribute arguments must be in parentheses: `sscanf("")`
//  --> derive_struct_attributes.rs:2:3
//   |
// 2 | #[sscanf = ""]
//   |   ^^^^^^^^^^^

///// TooManyAttributes /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "", format_regex = "")]
//~                   ^^^^^^^^^^^^^^^^^ cannot specify both `format` and `format_regex`
//~      ^^^^^^^^^^^ cannot specify both `format` and `format_regex`
struct TestTooManyAttributes;

///// TooManyAttributes2 /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "", format_regex = "", transparent)]
//~                                      ^^^^^^^^^^^ only one of `format`, `format_regex`, or `transparent` is allowed
//~                   ^^^^^^^^^^^^^^^^^ only one of `format`, `format_regex`, or `transparent` is allowed
//~      ^^^^^^^^^^^ only one of `format`, `format_regex`, or `transparent` is allowed
struct TestTooManyAttributes2;

///// MultipleAttributes /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
//~      ^^^^^^^^^^^ previous use here
#[sscanf(format = "")]
//~      ^^^^^^^^^^^ attribute `format` is specified multiple times
struct TestMultipleAttributes;

///// MultipleDifferentAttributes /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
//~      ^^^^^^^^^^^ cannot specify both `format` and `format_regex`
#[sscanf(format_regex = "")]
//~      ^^^^^^^^^^^^^^^^^ cannot specify both `format` and `format_regex`
struct TestMultipleDifferentAttributes;

///// InvalidAttribute /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "", bob = "")]
//~                   ^^^ unknown attribute `bob`. Valid attributes are: `format`, `format_regex`, or `transparent`
struct TestInvalidAttribute;

///// MultipleInvalidAttribute /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
#[sscanf(bob = "")]
//~      ^^^ unknown attribute `bob`. Valid attributes are: `format`, `format_regex`, or `transparent`
struct TestMultipleInvalidAttribute;

///// MissingComma /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "" bob = "")]
//~                  ^^^ expected `,`
struct TestMissingComma;

///// FormatNotString /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = 5)]
//~               ^ expected string literal
struct TestFormatNotString;

///// FormatMissingValue /////
#[derive(sscanf::FromScanf)]
#[sscanf(format =)]
//~              ^ expected an expression after `=`
struct TestFormatMissingValue;

///// FormatMissingEquals /////
#[derive(sscanf::FromScanf)]
#[sscanf(format "")]
//~             ^^ expected `,` or `=`
struct TestFormatMissingEquals;

///// FormatJustIdent /////
#[derive(sscanf::FromScanf)]
#[sscanf(format)]
//~      ^^^^^^ attribute `format` has the format: `#[sscanf(format = "<format>")]`
//~             where `<format>` is a format string using the field names inside of its placeholders
struct TestFormatJustIdent;

///// FormatMissingName /////
#[derive(sscanf::FromScanf)]
#[sscanf(= "")]
//~      ^ expected identifier
struct TestFormatMissingName;

///// DefaultOnStruct /////
#[derive(sscanf::FromScanf)]
#[sscanf(default)]
//~      ^^^^^^^ attribute `default` can only be used on fields.
//~              structs can have the following attributes: `format`, `format_regex`, or `transparent`
struct TestDefaultOnStruct;

///// DefaultWithValueOnStruct /////
#[derive(sscanf::FromScanf)]
#[sscanf(default = "")]
//~      ^^^^^^^ attribute `default` can only be used on fields.
//~              structs can have the following attributes: `format`, `format_regex`, or `transparent`
struct TestDefaultWithValueOnStruct;

///// InvalidIdent /////
#[derive(sscanf::FromScanf)]
#[sscanf(bob)]
//~      ^^^ unknown attribute `bob`. Valid attributes are: `format`, `format_regex`, or `transparent`
struct TestInvalidIdent;

///// TypoInIdent /////
#[derive(sscanf::FromScanf)]
#[sscanf(formt)]
//~      ^^^^^ unknown attribute `formt`. Did you mean `format`?
struct TestTypoInIdent;

///// MoreTyposInIdent /////
#[derive(sscanf::FromScanf)]
#[sscanf(formad_reggae)]
//~      ^^^^^^^^^^^^^ unknown attribute `formad_reggae`. Did you mean `format_regex`?
struct TestMoreTyposInIdent;

///// TyposAndWrongIdent /////
#[derive(sscanf::FromScanf)]
#[sscanf(defauld)]
//~      ^^^^^^^ unknown attribute `defauld` is similar to `default`, which can only be used on fields.
//~              structs can have the following attributes: `format`, `format_regex`, or `transparent`
struct TestTyposAndWrongIdent;

///// TransparentNoField /////
#[derive(sscanf::FromScanf)]
#[sscanf(transparent)]
//~      ^^^^^^^^^^^ structs or variants marked as `transparent` must have exactly one field
struct TestTransparentNoField;

///// TransparentMultiField /////
#[derive(sscanf::FromScanf)]
#[sscanf(transparent)]
//~      ^^^^^^^^^^^ structs or variants marked as `transparent` must have exactly one field
struct TestTransparentMultiField(usize, u8);

///// TransparentArg /////
#[derive(sscanf::FromScanf)]
#[sscanf(transparent(5))]
//~                 ^ expected `,` or `=`
struct TestTransparentArg(usize);

///// TransparentValue /////
#[derive(sscanf::FromScanf)]
#[sscanf(transparent = "true")]
//~                    ^^^^^^ attribute `transparent` does not take a value
struct TestTransparentValue(usize);
