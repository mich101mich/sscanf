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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn succeeding_parses() {
        macro_rules! assert_re_eq {
            ( $input:literal, $regex:literal ) => {
                let src = str_lit! { $input };
                let mut parser = FormatStringParser::new(src.to_slice());
                let r: RegexOverride = parser.parse().expect("expected parse to succeed");

                let mut leftover = String::new();
                while let Ok((_, c)) = parser.take() {
                    leftover.push(c);
                }
                if !leftover.is_empty() {
                    panic!(
                        "expected src {:?} to be fully consumed, but leftover: {:?}",
                        src.to_slice().text(),
                        leftover
                    );
                }

                if r.regex != $regex {
                    panic!(
                        "expected src {:?} to parse into\n{:?}\n but got\n{:?}",
                        src.to_slice().text(),
                        $regex,
                        r.regex
                    );
                }
            };
        }

        assert_re_eq!("/abc/", r#"abc"#);

        assert_re_eq!("/a\\/b/", r#"a/b"#); // '/a\/b/'
        assert_re_eq!(r#"/a\/b/"#, r#"a/b"#);

        assert_re_eq!("/a\\b/", r#"a\b"#); // '/a\b/' => '\b' is a regex boundary
        assert_re_eq!(r#"/a\b/"#, r#"a\b"#);

        assert_re_eq!(
            "/a\nb/", r#"a
b"#
        ); // '/a\nb/'
        assert_re_eq!("/a\\nb/", r#"a\nb"#); // '/a\nb/'

        assert_re_eq!("/a\\\\b/", r#"a\\b"#); // '/a\\b/'
    }

    #[test]
    fn failing_parses() {
        macro_rules! assert_re_err {
            ( $input:literal, $msg:literal ) => {
                let src = str_lit! { $input };
                let mut parser = FormatStringParser::new(src.to_slice());
                let r: Result<RegexOverride> = parser.parse();
                match r {
                    Ok(r) => panic!(
                        "expected src {:?} to fail parsing with message {:?}, but got regex {:?}",
                        src.to_slice().text(),
                        $msg,
                        r.regex
                    ),
                    Err(err) => {
                        let err_msg = format!("{}", err);
                        if err_msg != *$msg {
                            panic!(
                                "expected src {:?} to fail parsing with message\n{:?}\nbut got error message\n{:?}",
                                src.to_slice().text(),
                                $msg,
                                err_msg
                            );
                        }
                    }
                }
            };
        }

        assert_re_err!(
            "/abc",
            r#"missing '/' to close the regex option:
   --> /inside/a/test.rs:999:124
    |
999 | "/abc"
    |  ^^^^
"#
        );

        assert_re_err!(
            "/a\\/",
            r#"missing unescaped '/' to close the regex option:
   --> /inside/a/test.rs:999:124
    |
999 | "/a\\/"
    |  ^^^^^
"#
        );

        assert_re_err!(
            "/a\\",
            r#"missing '/' to close the regex option:
   --> /inside/a/test.rs:999:124
    |
999 | "/a\\"
    |  ^^^^
"#
        );

        assert_re_err!(
            "/a\\V/",
            r#"invalid regex override: regex parse error:
    a\V
     ^^
error: unrecognized escape sequence:
   --> /inside/a/test.rs:999:124
    |
999 | "/a\\V/"
    |  ^^^^^^
"#
        );
    }
}
