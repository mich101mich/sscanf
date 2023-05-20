use super::*;

pub type StructAttribute = SingleAttributeContainer<attr::Struct, StructAttributeKind>;

pub enum StructAttributeKind {
    Format { value: StrLit, escape: bool },
    Transparent,
}

impl FromAttribute<attr::Struct> for StructAttributeKind {
    fn from_attribute(attr: Attribute<attr::Struct>, _: ()) -> Result<Self> {
        let ret = match attr.kind {
            attr::Struct::Format | attr::Struct::FormatUnescaped => {
                let value = attr.value_as(
                    "\"<format>\"",
                    Some("where `<format>` is a format string using the field names inside of its placeholders")
                )?; // checked in tests/fail/derive_struct_attributes.rs
                Self::Format {
                    value,
                    escape: attr.kind != attr::Struct::FormatUnescaped,
                }
            }
            attr::Struct::Transparent => {
                if let Some(value) = attr.value.as_ref() {
                    let msg = format!("attribute `{}` does not take a value", attr.kind);
                    return Error::err_spanned(value, msg); // checked in tests/fail/derive_struct_attributes.rs
                }
                Self::Transparent
            }
        };
        Ok(ret)
    }
}
