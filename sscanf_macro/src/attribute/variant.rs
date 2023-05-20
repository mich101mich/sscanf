use super::*;

pub type VariantAttribute = SingleAttributeContainer<attr::Variant, VariantAttributeKind>;

pub enum VariantAttributeKind {
    StructLike(StructAttributeKind),
    Skip,
}

impl FromAttribute<attr::Variant> for VariantAttributeKind {
    fn from_attribute(attr: Attribute<attr::Variant>, _: ()) -> Result<Self> {
        let struct_kind = match attr.kind {
            attr::Variant::Skip => return Ok(Self::Skip),
            attr::Variant::Format => attr::Struct::Format,
            attr::Variant::FormatUnescaped => attr::Struct::FormatUnescaped,
            attr::Variant::Transparent => attr::Struct::Transparent,
        };
        let mapped_attr = Attribute {
            src: attr.src,
            kind: struct_kind,
            value: attr.value,
        };
        let kind = StructAttributeKind::from_attribute(mapped_attr, ())?;
        Ok(Self::StructLike(kind))
    }
}
