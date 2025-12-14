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
        let mut was_escape = false;
        let mut had_escaped_slash = false;
        loop {
            let Ok((_, c)) = parser.take() else {
                let msg = if had_escaped_slash {
                    "missing unescaped '/' to close the regex option"
                } else {
                    "missing '/' to close the regex option"
                };
                return parser.err_since(start, msg);
            };
            if c == '/' {
                if was_escape {
                    regex.push('/'); // escaped slash => keep the slash
                    had_escaped_slash = true;
                    was_escape = false;
                } else {
                    break;
                }
            } else if c == '\\' {
                if was_escape {
                    regex.push('\\'); // escaped backslash => keep both, because regex will escape it again
                    regex.push('\\');
                    was_escape = false;
                } else {
                    was_escape = true;
                }
            } else {
                if was_escape {
                    regex.push('\\'); // some other escaped char => keep the backslash and the char
                    was_escape = false;
                }
                regex.push(c);
            }
        }
        let src = parser.slice_since(start);

        if let Err(err) = regex_syntax::Parser::new().parse(&regex) {
            bail!(src => "invalid regex override: {}", err);
        }

        Ok(Self { src, regex })
    }
}

impl ErrorTarget for RegexOverride<'_> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}
