use super::*;

use std::fmt::Write;

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
                let mut closure_hint = String::from(
                    "where `<type>` is the type that should be matched against and `<conversion>` converts from `<type>` to `",
                );
                if filters {
                    write!(closure_hint, "Option<{}>", ty.to_token_stream()).unwrap();
                } else {
                    write!(closure_hint, "{}", ty.to_token_stream()).unwrap();
                }
                closure_hint.push('`');

                let mapper = attr.value_as::<syn::Expr>(closure_format, Some(&closure_hint))?;
                let syn::Expr::Closure(mapper) = mapper else {
                    bail!(mapper => "attribute `{}` requires a closure like: `{closure_format}`\n{closure_hint}", attr.kind);
                };

                let param = if mapper.inputs.len() == 1 {
                    mapper.inputs.first().unwrap() // safe because len() == 1
                } else {
                    let mut span_src = TokenStream::new();
                    for param in mapper.inputs.pairs().skip(1) {
                        param.to_tokens(&mut span_src);
                    }
                    if span_src.is_empty() {
                        // no arguments were given => point to the empty `||`
                        mapper.or1_token.to_tokens(&mut span_src);
                        mapper.or2_token.to_tokens(&mut span_src);
                    }
                    bail!(span_src => "attribute `{}` requires a closure with exactly one argument", attr.kind);
                };

                let ty = if let syn::Pat::Type(ty) = param {
                    (*ty.ty).clone()
                } else {
                    bail!(param => "`{}` closure has to specify the type of the argument", attr.kind);
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
                    if attr.kind == attr::Field::From {
                        "Into"
                    } else {
                        "TryInto"
                    },
                    ty.to_token_stream()
                );
                // can't convert directly to `syn::Type` because error messages would be confusing
                let ty = attr.value_as::<Type>("<type>", Some(&hint))?;

                Self::From {
                    ty: ty.into_inner(),
                    tries: attr.kind == attr::Field::TryFrom,
                }
            }
        };
        Ok(ret)
    }
}
