///// Test1 /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{5} {x} {} {} {b} {} {} {b}")]
//~               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ placeholder `b` is used multiple times in the format string:
//~                                              --> stable/derive_placeholders.rs:2:44
//~                                               |
//~                                             2 | "{5} {x} {} {} {b} {} {} {b}"
//~                                               |                          ^^^
//~               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ placeholder `b` is used multiple times in the format string:
//~                                              --> stable/derive_placeholders.rs:2:34
//~                                               |
//~                                             2 | "{5} {x} {} {} {b} {} {} {b}"
//~                                               |                ^^^
struct Test1 {
    a: u8,
    b: u8,
}

///// Test2 /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{b}")]
//~               ^^^^^ field `b` is marked as default, but is also used in a placeholder:
//~                      --> stable/derive_placeholders.rs:2:20
//~                       |
//~                     2 | "{b}"
//~                       |  ^^^
struct Test2 {
    a: u8,
    #[sscanf(default = 5)]
    //~      ^^^^^^^^^^^ field `b` is marked as default, but is also used in a placeholder
    b: u8,
}
