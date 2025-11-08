use std::borrow::Cow;

#[derive(sscanf::FromScanf)]
#[sscanf("{x} {y} {z}")]
struct Test<'a> {
    x: &'static str,
    y: Cow<'a, str>,
    z: Cow<'static, str>,
}
