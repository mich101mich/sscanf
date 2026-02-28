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
            $($ident,)+
            $($special_ident,)+
        }
        impl AutoGenKind {
            const AUTOGEN_KINDS: &[&str] = &[
                $($text,)+
                $($special_text,)+
            ];

            pub fn valid_hint() -> String {
                list_items_quoted(Self::AUTOGEN_KINDS, '"')
            }

            pub fn from_str(s: &str) -> Result<Self> {
                match s {
                    $($text => Ok(Self::$ident),)+
                    $(s if $matching(s) => Ok(Self::$special_ident),)+
                    _ => {
                        if let Some(closest) = find_closest(s, Self::AUTOGEN_KINDS) {
                            bail!(s => r#"invalid value for autogen: "{s}". Did you mean "{closest}"?"#);
                        } else {
                            bail!(s => r#"invalid value for autogen: "{s}". valid values are: {}"#, Self::valid_hint());
                        }
                    }
                }
            }

            pub fn create_struct_attr(&self, field_name: &str, src: TokenStream) -> StructAttribute {
                let (matched_text, escape) = match *self {
                    $(Self::$ident => (field_name.to_case(Case::$case), true),)+
                    $(Self::$special_ident => $conversion(field_name),)+
                };
                let value = StrLit::from_parts(&matched_text, src.span());
                let kind = StructAttributeKind::Format { value, escape };
                StructAttribute::new(src, kind)
            }
        }
    };
}

fn match_case_sensitive(s: &str) -> bool {
    s == "CaseSensitive"
}
fn convert_case_sensitive(ident: &str) -> (String, bool) {
    (ident.to_string(), true)
}
fn match_case_insensitive(s: &str) -> bool {
    s.to_case(Case::Flat) == "caseinsensitive"
}
fn convert_case_insensitive(ident: &str) -> (String, bool) {
    (format!("(?i:{ident})"), false) // don't escape, since we are adding regex syntax
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
        UpperSnakeCase: ("UPPER_SNAKE_CASE", UpperSnake),
        ScreamingSnakeCase: ("SCREAMING_SNAKE_CASE", UpperSnake),
        KebabCase: ("kebab-case", Kebab),
        UpperKebabCase: ("UPPER-KEBAB-CASE", UpperKebab),
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

        let value = attr.value_as::<syn::LitStr>("\"<casing>\"", Some(&casing_hint))?;
        Self::from_str(&value.value())
    }
}

pub type EnumAttribute = SingleAttributeContainer<attr::Enum, EnumAttributeKind>;

pub enum EnumAttributeKind {
    AutoGen(AutoGenKind),
}

impl FromAttribute<attr::Enum> for EnumAttributeKind {
    fn from_attribute(attr: Attribute<attr::Enum>, (): ()) -> Result<Self> {
        let ret = match attr.kind {
            attr::Enum::AutoGen | attr::Enum::AutoGenerate => {
                let kind = AutoGenKind::from_attr(&attr)?;
                Self::AutoGen(kind)
            }
        };
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use AutoGenKind::*;

    #[track_caller]
    fn parse(s: &str) -> AutoGenKind {
        AutoGenKind::from_str(s).unwrap()
    }

    #[test]
    fn autogen_from_str() {
        assert_eq!(parse("lower case"), LowerCase);
        assert_eq!(parse("UPPERCASE"), UpperFlatCase);
        assert_eq!(parse("CaseSensitive"), CaseSensitive);
        assert_eq!(parse("caseinsensitive"), CaseInsensitive);
    }

    #[test]
    fn autogen_from_str_invalid() {
        let err = AutoGenKind::from_str("unknown").unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("invalid value for autogen"));
        assert!(err_msg.contains("valid values are"));

        AutoGenKind::from_str("uppercase").unwrap_err(); // not UPPERCASE
        AutoGenKind::from_str("camel_case").unwrap_err(); // not camelCase
    }

    #[test]
    fn case_insensitive_matching() {
        assert!(match_case_insensitive("CaseInsensitive"));
        assert!(match_case_insensitive("caseinsensitive"));
        assert!(match_case_insensitive("CASEINSENSITIVE"));
        assert!(match_case_insensitive("Case Insensitive"));
        assert!(match_case_insensitive("Case_Insensitive"));
        assert!(match_case_insensitive("cAsEiNsEnSiTiVe"));
        assert!(!match_case_insensitive("Case Sensitive"));
    }
}
