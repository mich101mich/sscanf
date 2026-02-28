use super::*;

pub type StructAttribute = SingleAttributeContainer<attr::Struct, StructAttributeKind>;

pub enum StructAttributeKind {
    Format { value: StrLit, escape: bool },
    Transparent,
}

impl FromAttribute<attr::Struct> for StructAttributeKind {
    fn from_attribute(attr: Attribute<attr::Struct>, (): ()) -> Result<Self> {
        let ret = match attr.kind {
            attr::Struct::Format | attr::Struct::FormatRegex => {
                let value = attr.value_as(
                    "\"<format>\"",
                    Some("where `<format>` is a format string using the field names inside of its placeholders")
                )?;
                Self::Format {
                    value,
                    escape: attr.kind != attr::Struct::FormatRegex,
                }
            }
            attr::Struct::Transparent => {
                if let Some(value) = attr.value.as_ref() {
                    bail!(value => "attribute `{}` does not take a value", attr.kind);
                }
                Self::Transparent
            }
        };
        Ok(ret)
    }
}
