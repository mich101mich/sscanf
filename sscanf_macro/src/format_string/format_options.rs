use crate::*;

mod parse;
mod print;

/// Replica of sscanf::advanced::FormatOptions, but with an additional `regex` field
pub struct FormatOptions<'a> {
    pub src: StrLitSlice<'a>,
    pub regex: Option<RegexOverride<'a>>,
    pub number: Option<NumberFormatOption>,
    pub custom: Option<CustomFormatOption<'a>>,
}

pub enum NumberFormatOption {
    Binary(NumberPrefixPolicy),
    Octal(NumberPrefixPolicy),
    Decimal,
    Hexadecimal(NumberPrefixPolicy),
    Other(u32),
}

pub enum NumberPrefixPolicy {
    Forbidden,
    Optional,
    Required,
}

pub struct RegexOverride<'a> {
    #[allow(unused, reason = "TODO: used once other todos are done")]
    pub src: StrLitSlice<'a>,
    pub regex: String,
}

pub struct CustomFormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub custom: String,
}
