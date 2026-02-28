///// NoValue /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoValue(#[sscanf(default =)] u8);
//~                                  ^ expected an expression after `=`

///// NoEquals /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoEquals(#[sscanf(default 5)] u8);
//~                                  ^ expected `,` or `=`

///// NoIdent /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoIdent(#[sscanf(= 5)] u8);
//~                         ^ expected identifier

///// FormatInField /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestFormatInField(#[sscanf(format = "")] u8);
//~                               ^^^^^^ attribute `format` can only be used on structs or variants.
//~                                      fields can have the following attributes: `default`, `map`, `filter_map`, `from`, or `try_from`

///// ImpliedFormatInField /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestImpliedFormatInField(#[sscanf("")] u8);
//~                                      ^ omitting the attribute name is only valid for the `format` attribute on structs or variants

///// UnknownArg /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestUnknownArg(#[sscanf(bob = 5)] u8);
//~                            ^^^ unknown attribute `bob`. Valid attributes are: `default`, `map`, `filter_map`, `from`, or `try_from`

///// TypoInArg /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestTypoInArg(#[sscanf(mao)] u8);
//~                           ^^^ unknown attribute `mao`. Did you mean `map`?

///// MoreTyposInArg /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestMoreTyposInArg(#[sscanf(defold)] u8);
//~                                ^^^^^^ unknown attribute `defold`. Did you mean `default`?

///// DuplicateArg /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDuplicateArg(#[sscanf(default = 5, default = 5)] u8);
//~                                           ^^^^^^^^^^^ attribute `default` is specified multiple times
//~                              ^^^^^^^^^^^ previous use here

///// DuplicateMultiArg /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDuplicateMultiArg(
    #[sscanf(default = 5)]
    //~      ^^^^^^^^^^^ previous use here
    #[sscanf(default = 5)]
    //~      ^^^^^^^^^^^ attribute `default` is specified multiple times
    u8,
);

///// DefaultAndMap /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultAndMap(#[sscanf(default = 5, map = |x: usize| { x as u8 })] u8);
//~                                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot specify both `default` and `map`
//~                               ^^^^^^^^^^^ cannot specify both `default` and `map`

///// DefaultAndMapMulti /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultAndMapMulti(
    #[sscanf(default = 5)]
    //~      ^^^^^^^^^^^ cannot specify both `default` and `map`
    #[sscanf(map = |x: usize| { x as u8 })]
    //~      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot specify both `default` and `map`
    u8,
);

///// NoPlaceholder /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestNoPlaceholder(#[sscanf(map = |x: usize| { x as u8 })] u8);
//~                                                              ^^ Field `0` is not specified in the format string.
//~                                                                 Either add more placeholders or provide a default value with `#[sscanf(default)]` or `#[sscanf(default = ...)]`

///// MapNoAssign /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoAssign(#[sscanf(map)] u8);
//~                             ^^^ attribute `map` has the format: `#[sscanf(map = |<arg>: <type>| <conversion>)]`
//~                                 where `<type>` is the type that should be matched against and `<conversion>` converts from `<type>` to `u8`

///// MapNoValue /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoValue(#[sscanf(map =)] u8);
//~                                 ^ expected an expression after `=`

///// MapNotClosure /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNotClosure(#[sscanf(map = "")] u8);
//~                                     ^^ attribute `map` requires a closure like: `|<arg>: <type>| <conversion>`
//~                                        where `<type>` is the type that should be matched against and `<conversion>` converts from `<type>` to `u8`

///// MapNoParam /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoParam(#[sscanf(map = || { x as u8 })] u8);
//~                                  ^^ attribute `map` requires a closure with exactly one argument

///// MapNoType /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapNoType(#[sscanf(map = |x| { x as u8 })] u8);
//~                                  ^ `map` closure has to specify the type of the argument

///// MapMoreTypes /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapMoreTypes(#[sscanf(map = |x: usize, y: usize| { x as u8 })] u8);
//~                                               ^^^^^^^^ attribute `map` requires a closure with exactly one argument

///// FilterMapNoOption /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFilterMapNoOption(#[sscanf(filter_map = |x: usize| { x as u8 })] u8);
//~                                                             ^^^^^^^ error: mismatched types
//~                                                                     label: expected `Option<u8>`, found `u8`

///// FilterMapNoValue /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFilterMapNoValue(#[sscanf(filter_map)] u8);
//~                                  ^^^^^^^^^^ attribute `filter_map` has the format: `#[sscanf(filter_map = |<arg>: <type>| <conversion>)]`
//~                                             where `<type>` is the type that should be matched against and `<conversion>` converts from `<type>` to `Option<u8>`

///// FromNoType /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFromNoType(#[sscanf(from)] u8);
//~                            ^^^^ attribute `from` has the format: `#[sscanf(from = <type>)]`
//~                                 where `<type>` is the type that should be matched against and implements `Into<u8>`

///// FromNotType /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFromNotType(#[sscanf(from = "")] u8);
//~                                    ^^ expected identifier

///// TryFromNoType /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestTryFromNoType(#[sscanf(try_from)] u8);
//~                               ^^^^^^^^ attribute `try_from` has the format: `#[sscanf(try_from = <type>)]`
//~                                        where `<type>` is the type that should be matched against and implements `TryInto<u8>`

///// DefaultWrongType /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultWrongType(#[sscanf(default = "")] u8);
//~                                            ^^ error: mismatched types
//~                                               label: expected `u8`, found `&str`

///// MapWrongReturn /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestMapWrongReturn(#[sscanf(map = |x: usize| { x })] u8);
//~                                                   ^ error: mismatched types
//~                                                     label: expected `u8`, found `usize`

///// DefaultNoDefault /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
struct TestDefaultNoDefault(#[sscanf(default)] std::num::ParseIntError);
//~                                            ^^^^^^^^^^^^^^^^^^^^^^^ error: the trait bound `ParseIntError: Default` is not satisfied
//~                                                                    label: the trait `Default` is not implemented for `ParseIntError`

///// FromNoFrom /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "{}")]
struct TestFromNoFrom(#[sscanf(from = f32)] u8);
//~                                   ^^^ error: the trait bound `u8: From<f32>` is not satisfied
//~                                       label: the trait `From<f32>` is not implemented for `u8`
