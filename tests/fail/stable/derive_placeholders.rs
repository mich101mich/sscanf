///// multiple placeholders /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {b} {a}")]
//~               ^^^^^^^^^^^^^ placeholder `a` is used multiple times in the format string:
//~                              --> stable/derive_placeholders.rs:2:28
//~                               |
//~                             2 | "{a} {b} {a}"
//~                               |          ^^^
//~               ^^^^^^^^^^^^^ placeholder `a` is used multiple times in the format string:
//~                              --> stable/derive_placeholders.rs:2:20
//~                               |
//~                             2 | "{a} {b} {a}"
//~                               |  ^^^
struct Test {
    a: u8,
    b: u8,
}

///// name and index /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {b} {0}")]
//~               ^^^^^^^^^^^^^ field `a` is used in multiple placeholders:
//~                              --> stable/derive_placeholders.rs:2:28
//~                               |
//~                             2 | "{a} {b} {0}"
//~                               |          ^^^
//~               ^^^^^^^^^^^^^ field `a` is used in multiple placeholders:
//~                              --> stable/derive_placeholders.rs:2:20
//~                               |
//~                             2 | "{a} {b} {0}"
//~                               |  ^^^
struct Test {
    a: u8,
    b: u8,
}

///// name and default /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{b}")]
//~               ^^^^^ field `b` is marked as default, but is also used in a placeholder:
//~                      --> stable/derive_placeholders.rs:2:20
//~                       |
//~                     2 | "{b}"
//~                       |  ^^^
struct Test {
    a: u8,
    #[sscanf(default = 5)]
    //~      ^^^^^^^^^^^ field `b` is marked as default, but is also used in a placeholder
    b: u8,
}

///// index and default /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{1}")]
//~               ^^^^^ field `b` is marked as default, but is also used in a placeholder:
//~                      --> stable/derive_placeholders.rs:2:20
//~                       |
//~                     2 | "{1}"
//~                       |  ^^^
struct Test {
    a: u8,
    #[sscanf(default = 5)]
    //~      ^^^^^^^^^^^ field `b` is marked as default, but is also used in a placeholder
    b: u8,
}

///// tuple index and default /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{1}")]
//~               ^^^^^ field `1` is marked as default, but is also used in a placeholder:
//~                      --> stable/derive_placeholders.rs:2:20
//~                       |
//~                     2 | "{1}"
//~                       |  ^^^
struct Test(u8, #[sscanf(default = 5)] u8);
//~                      ^^^^^^^^^^^ field `1` is marked as default, but is also used in a placeholder

///// field not used /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a}")]
struct Test {
    a: u8,
    b: u8,
}

//~~~~~~~~~~~~~~~~~~~~ errors ~~~~~~~~~~~~~~~~~~~~//

// error: Field `b` is not specified in the format string.
//        Either add more placeholders or provide a default value with `#[sscanf(default)]` or `#[sscanf(default = ...)]`
//  --> stable/derive_placeholders.rs:5:5
//   |
// 5 |     b: u8,
//   |     ^

///// tuple field not used /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{0}")]
struct Test(u8, u8);
//~             ^^ Field `1` is not specified in the format string.
//~                Either add more placeholders or provide a default value with `#[sscanf(default)]` or `#[sscanf(default = ...)]`

///// extra placeholder /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {b} {}")]
//~               ^^^^^^^^^^^^ More placeholders than fields in the format string:
//~                             --> stable/derive_placeholders.rs:2:28
//~                              |
//~                            2 | "{a} {b} {}"
//~                              |          ^^
struct Test {
    a: u8,
    b: u8,
}

///// extra field /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {b} {c}")]
//~               ^^^^^^^^^^^^^ placeholder `c` does not match any field:
//~                              --> stable/derive_placeholders.rs:2:28
//~                               |
//~                             2 | "{a} {b} {c}"
//~                               |          ^^^
struct Test {
    a: u8,
    b: u8,
}

///// extra index /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {b} {2}")]
//~               ^^^^^^^^^^^^^ No field with index 2 exists:
//~                              --> stable/derive_placeholders.rs:2:28
//~                               |
//~                             2 | "{a} {b} {2}"
//~                               |          ^^^
struct Test {
    a: u8,
    b: u8,
}

///// typo field /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {teh_field_name}")]
//~               ^^^^^^^^^^^^^^^^^^^^^^ placeholder `teh_field_name` does not match any field. Did you mean `the_field_name`?:
//~                                       --> stable/derive_placeholders.rs:2:24
//~                                        |
//~                                      2 | "{a} {teh_field_name}"
//~                                        |      ^^^^^^^^^^^^^^^^
struct Test {
    a: u8,
    the_field_name: u8,
}

///// typo field duplicate /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{a} {the_field_name} {teh_field_name}")]
//~               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ placeholder `teh_field_name` does not match any field:
//~                                                        --> stable/derive_placeholders.rs:2:41
//~                                                         |
//~                                                       2 | "{a} {the_field_name} {teh_field_name}"
//~                                                         |                       ^^^^^^^^^^^^^^^^
struct Test {
    a: u8,
    the_field_name: u8,
}
