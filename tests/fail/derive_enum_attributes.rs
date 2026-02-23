///// OuterFormat /////
#[derive(sscanf::FromScanf)]
#[sscanf(format = "")]
//~      ^^^^^^ attribute `format` can only be used on structs or variants.
//~             enums can have the following attributes: `autogen` or `autogenerate`
enum TestOuterFormat { A, B }

///// NoFormat /////
#[derive(sscanf::FromScanf)]
enum TestNoFormat { A, B }
//~  ^^^^^^^^^^^^ at least one variant has to be constructable from sscanf.
//~               To do this, add #[sscanf(format = "...")] to a variant

///// NoVariants /////
#[derive(sscanf::FromScanf)]
enum TestNoVariants {}
//~  ^^^^^^^^^^^^^^ FromScanf: enums must have at least one variant

///// AutogenHasFields /////
#[derive(sscanf::FromScanf)]
#[sscanf(autogen)]
enum TestAutogenHasFields { A, B(usize) }
//~                             ^^^^^^^ FromScanf: autogen only works if the variants have no fields.
//~                                     Use `#[sscanf(format = "...")]` to specify a format for a variant with fields or `#[sscanf(skip)]` to skip a variant

///// AutogenAllSkip /////
#[derive(sscanf::FromScanf)]
#[sscanf(autogen)]
enum TestAutogenAllSkip { #[sscanf(skip)] A, #[sscanf(skip)] B }
//~  ^^^^^^^^^^^^^^^^^^ at least one variant has to be constructable from sscanf and not skipped.

///// AutogenInvalid /////
#[derive(sscanf::FromScanf)]
#[sscanf(autogen = {})]
//~                ^^ expected string literal
enum TestAutogenInvalid { A, B }

///// AutogenInvalidWord /////
#[derive(sscanf::FromScanf)]
//~      ^^^^^^^^^^^^^^^^^ invalid value for autogen: "bob". valid values are: "lower case", "UPPER CASE", "lowercase", "UPPERCASE", "PascalCase", "camelCase", "snake_case", "UPPER_SNAKE_CASE", "SCREAMING_SNAKE_CASE", "kebab-case", "UPPER-KEBAB-CASE", "SCREAMING-KEBAB-CASE", "CaseSensitive", or "CaseInsensitive"
#[sscanf(autogen = "bob")]
enum TestAutogenInvalidWord { A, B }

///// AutogenInvalidCase /////
#[derive(sscanf::FromScanf)]
//~      ^^^^^^^^^^^^^^^^^ invalid value for autogen: "casesensitive". Did you mean "CaseInsensitive"?
#[sscanf(autogen = "casesensitive")]
enum TestAutogenInvalidCase { A, B }
