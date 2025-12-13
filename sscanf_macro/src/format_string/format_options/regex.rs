use super::*;

#[derive(Clone)]
pub struct RegexOverride<'a> {
    pub src: StrLitSlice<'a>,
    pub regex: String,
}

impl<'a> FromFormatString<'a> for RegexOverride<'a> {
    /// "...{<ident>:.../<regex>/...}..."
    ///                 ^parser  ^parser when done
    fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let (start, start_slash) = parser.take()?;
        assert_eq!(start_slash, '/');

        let mut regex = String::new();
        let mut escape = None; // index of the last '\\', if any
        loop {
            let Ok((i, c)) = parser.take() else {
                return parser.err_since(start, "missing '/' to close the regex option");
            };
            if c == '/' {
                if escape.take().is_some() {
                    regex.push('/');
                } else {
                    break;
                }
            } else if c == '\\' {
                if escape.take().is_some() {
                    regex.push('\\');
                    regex.push('\\');
                } else {
                    escape = Some(i);
                }
            } else {
                if escape.take().is_some() {
                    regex.push('\\');
                }
                regex.push(c);
            }
        }
        let src = parser.slice_since(start);
        Ok(Self { src, regex })
    }
}

impl ErrorTarget for RegexOverride<'_> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
