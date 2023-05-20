use super::*;

use convert_case::{Case, Casing};

macro_rules! declare_autogen {
    (
        normal: {
            $($ident: ident : ( $text: literal, $case: ident ),)+
        },
        special: {
            $($special_ident: ident : ( $special_text: literal, $matching: ident, $conversion: ident ),)+
        },
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum AutoGenKind {
            $($special_ident,)+
            $($ident,)+
        }
        impl AutoGenKind {
            const AUTOGEN_KINDS: &'static [&'static str] = &[$($text,)+ $($special_text,)+];

            pub fn valid_hint() -> String {
                list_items(Self::AUTOGEN_KINDS, |kind| format!("\"{}\"", kind))
            }

            pub fn from_str(s: &str) -> Result<Self> {
                match s {
                    $($text => Ok(Self::$ident),)+
                    $(s if $matching(s) => Ok(Self::$special_ident),)+
                    _ => {
                        if let Some(closest) = find_closest(s, Self::AUTOGEN_KINDS) {
                            let msg = format!("invalid value for autogen: \"{}\". Did you mean \"{}\"?", s, closest);
                            Error::err_spanned(s, msg) // checked in tests/fail/derive_enum_attributes.rs
                        } else {
                            let msg = format!("invalid value for autogen: \"{}\". valid values are: {}", s, Self::valid_hint());
                            Error::err_spanned(s, msg) // checked in tests/fail/derive_enum_attributes.rs
                        }
                    }
                }
            }

            pub fn create_struct_attr(&self, ident: &str, src: TokenStream) -> StructAttribute {
                match *self {
                    $(Self::$special_ident => {
                        let kind = $conversion(ident, &src);
                        StructAttribute::new(src, kind)
                    },)+
                    $(Self::$ident => {
                        let lit = syn::LitStr::new(&ident.to_case(Case::$case), src.span());
                        let kind = StructAttributeKind::Format { value: StrLit::new(lit), escape: true };
                        StructAttribute::new(src, kind)
                    },)+
                }
            }
        }
    };
}

fn match_case_sensitive(s: &str) -> bool {
    s == "CaseSensitive"
}
fn convert_case_sensitive(ident: &str, src: &TokenStream) -> StructAttributeKind {
    StructAttributeKind::Format {
        value: StrLit::new(syn::LitStr::new(ident, src.span())),
        escape: true,
    }
}
fn match_case_insensitive(s: &str) -> bool {
    s.to_case(Case::Flat) == "caseinsensitive"
}
fn convert_case_insensitive(ident: &str, src: &TokenStream) -> StructAttributeKind {
    let text = format!("(?i:{})", ident);
    StructAttributeKind::Format {
        value: StrLit::new(syn::LitStr::new(&text, src.span())),
        escape: false,
    }
}

declare_autogen!(
    normal: {
        LowerCase: ("lower case", Lower),
        UpperCase: ("UPPER CASE", Upper),
        FlatCase: ("lowercase", Flat),
        UpperFlatCase: ("UPPERCASE", UpperFlat),
        PascalCase: ("PascalCase", Pascal),
        CamelCase: ("camelCase", Camel),
        SnakeCase: ("snake_case", Snake),
        ScreamingSnakeCase: ("SCREAMING_SNAKE_CASE", ScreamingSnake),
        KebabCase: ("kebab-case", Kebab),
        ScreamingKebabCase: ("SCREAMING-KEBAB-CASE", UpperKebab),
    },
    special: {
        CaseSensitive: ("CaseSensitive", match_case_sensitive, convert_case_sensitive),
        CaseInsensitive: ("CaseInsensitive", match_case_insensitive, convert_case_insensitive),
    },
);

impl AutoGenKind {
    fn from_attr(attr: &Attribute<attr::Enum>) -> Result<Self> {
        if attr.value.is_none() {
            return Ok(Self::CaseSensitive);
        }

        let casing_hint = format!("where `<casing>` is one of {}", AutoGenKind::valid_hint());

        let value = attr.value_as::<syn::LitStr>("\"<casing>\"", Some(&casing_hint))?; // checked in tests/fail/derive_enum_attributes.rs
        Self::from_str(&value.value())
    }
}

pub type EnumAttribute = SingleAttributeContainer<attr::Enum, EnumAttributeKind>;

pub enum EnumAttributeKind {
    AutoGen(AutoGenKind),
}

impl FromAttribute<attr::Enum> for EnumAttributeKind {
    fn from_attribute(attr: Attribute<attr::Enum>, _: ()) -> Result<Self> {
        let ret = match attr.kind {
            attr::Enum::AutoGen | attr::Enum::AutoGenerate => {
                let kind = AutoGenKind::from_attr(&attr)?;
                Self::AutoGen(kind)
            }
        };
        Ok(ret)
    }
}
