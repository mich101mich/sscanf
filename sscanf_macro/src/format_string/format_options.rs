use crate::*;

mod parse;
mod print;

/// Replica of sscanf::advanced::FormatOptions, but with an additional `regex` field
#[derive(Clone)]
pub struct FormatOptions<'a> {
    pub src: StrLitSlice<'a>,
    pub regex: Option<RegexOverride<'a>>,
    pub number: Option<NumberFormatOption>,
    pub custom: Option<CustomFormatOption<'a>>,
}

#[derive(Clone, Copy)]
pub enum NumberFormatOption {
    Binary(NumberPrefixPolicy),
    Octal(NumberPrefixPolicy),
    Decimal,
    Hexadecimal(NumberPrefixPolicy),
    Other(u32),
}

#[derive(Clone, Copy)]
pub enum NumberPrefixPolicy {
    Forbidden,
    Optional,
    Required,
}

#[derive(Clone)]
pub struct RegexOverride<'a> {
    #[allow(unused, reason = "TODO: used once other todos are done")]
    pub src: StrLitSlice<'a>,
    pub regex: String,
}

#[derive(Clone)]
pub struct CustomFormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub num_escapes: usize, // number of '#' characters before and after the custom format option
    pub custom: String,
}

impl<'a> Sourced<'a> for FormatOptions<'a> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
impl<'a> Sourced<'a> for RegexOverride<'a> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
impl<'a> Sourced<'a> for CustomFormatOption<'a> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
