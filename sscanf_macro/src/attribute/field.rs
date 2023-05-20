use super::*;

pub type FieldAttribute<'a> =
    SingleAttributeContainer<attr::Field, FieldAttributeKind, &'a syn::Type>;

pub enum FieldAttributeKind {
    Default(Option<syn::Expr>),
    Map {
        mapper: syn::ExprClosure,
        ty: syn::Type,
        filters: bool,
    },
    From {
        ty: syn::Type,
        tries: bool,
    },
}

impl FromAttribute<attr::Field, &'_ syn::Type> for FieldAttributeKind {
    fn from_attribute(attr: Attribute<attr::Field>, ty: &'_ syn::Type) -> Result<Self> {
        let ret = match attr.kind {
            attr::Field::Default => Self::Default(attr.value),
            attr::Field::Map | attr::Field::FilterMap => {
                let filters = attr.kind == attr::Field::FilterMap;

                let closure_format = "|<arg>: <type>| <conversion>";
                let mut closure_hint = String::from("where `<type>` is the type that should be matched against and `<conversion>` converts from `<type>` to `");
                if filters {
                    closure_hint.push_str(&format!("Option<{}>", ty.to_token_stream()));
                } else {
                    closure_hint.push_str(&ty.to_token_stream().to_string());
                }
                closure_hint.push('`');

                let mapper = attr.value_as::<syn::Expr>(closure_format, Some(&closure_hint))?; // checked in tests/fail/derive_field_attributes.rs
                let mapper = if let syn::Expr::Closure(closure) = mapper {
                    closure
                } else {
                    let msg = format!(
                        "attribute `{}` requires a closure like: `{}`\n{}",
                        attr.kind, closure_format, closure_hint
                    );
                    return Error::err_spanned(mapper, msg); // checked in tests/fail/derive_field_attributes.rs
                };

                let param = if mapper.inputs.len() == 1 {
                    mapper.inputs.first().unwrap()
                } else {
                    let msg = format!(
                        "attribute `{}` requires a closure with exactly one argument",
                        attr.kind
                    );
                    let mut span_src = TokenStream::new();
                    for param in mapper.inputs.pairs().skip(1) {
                        param.to_tokens(&mut span_src);
                    }
                    if span_src.is_empty() {
                        // no arguments were given => point to the empty `||`
                        mapper.or1_token.to_tokens(&mut span_src);
                        mapper.or2_token.to_tokens(&mut span_src);
                    }
                    return Error::err_spanned(span_src, msg); // checked in tests/fail/derive_field_attributes.rs
                };

                let ty = if let syn::Pat::Type(ty) = param {
                    (*ty.ty).clone()
                } else {
                    let msg = format!(
                        "`{}` closure has to specify the type of the argument",
                        attr.kind
                    );
                    return Error::err_spanned(param, msg); // checked in tests/fail/derive_field_attributes.rs
                };

                Self::Map {
                    mapper,
                    ty,
                    filters,
                }
            }
            attr::Field::From | attr::Field::TryFrom => {
                let hint = format!(
                    "where `<type>` is the type that should be matched against and implements `{}<{}>`",
                    if attr.kind == attr::Field::From { "Into" } else { "TryInto" },
                    ty.to_token_stream()
                );
                // can't convert directly to `syn::Type` because error messages would be confusing
                let ty = attr.value_as::<Type>("<type>", Some(&hint))?; // checked in tests/fail/derive_field_attributes.rs

                Self::From {
                    ty: ty.into_inner(),
                    tries: attr.kind == attr::Field::TryFrom,
                }
            }
        };
        Ok(ret)
    }
}
