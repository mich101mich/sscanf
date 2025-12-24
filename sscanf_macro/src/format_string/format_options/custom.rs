use super::*;

#[derive(Clone)]
pub struct CustomFormatOption<'a> {
    pub src: StrLitSlice<'a>,
    pub num_escapes: usize, // number of '#' characters before and after the custom format option
    pub content: String,
}

impl<'a> FromFormatString<'a> for CustomFormatOption<'a> {
    /// Parse a custom format option from the given parser
    ///
    /// "...{<ident>:...##[<custom>]##...}..."
    ///                 ^parser       ^parser when done
    fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let start = parser.get_pos();
        let mut num_escapes = 0;
        loop {
            let (pos, c) = parser.take()?;
            if c == '#' {
                num_escapes += 1;
            } else if c == '[' {
                break;
            } else {
                let msg = format!("expected `#` or `[` to start custom format option, found `{c}`");
                return parser.err_at(pos, msg);
            }
        }

        let mut content = String::new();
        let mut ending_escapes = None;
        loop {
            let Ok((_, c)) = parser.take() else {
                let sequence = "#".repeat(num_escapes);
                let msg = format!(
                    "Did not find the required ending sequence \"]{sequence}\" to end the custom format option"
                );
                return parser.err_since(start, msg);
            };
            if c == ']' {
                if num_escapes == 0 {
                    break; // finished parsing
                }
                ending_escapes = Some(num_escapes);
            } else if c == '#' {
                match ending_escapes {
                    None => {}        // no ']' found yet
                    Some(1) => break, // this was the last one => finished parsing
                    Some(ref mut v) => *v -= 1,
                }
            } else {
                ending_escapes = None; // reset because we found a non-`#` character
            }
            content.push(c);
        }

        // remove the ending ']' and '#' characters. Note that the last character was not pushed
        content.truncate(content.len() - num_escapes);

        let src = parser.slice_since(start);
        Ok(Self {
            src,
            num_escapes,
            content,
        })
    }
}

impl ToTokens for CustomFormatOption<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.src.span();
        let text = &self.content;
        tokens.extend(quote_spanned! {span=> ::std::option::Option::Some(::std::borrow::Cow::Borrowed(#text))});
    }
}

impl ErrorTarget for CustomFormatOption<'_> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_parse_result {
        ($input:literal, Ok($expected:literal)) => {{
            let src = str_lit! { $input };
            let mut parser = FormatStringParser::new(src.to_slice());
            let option = match parser.parse::<CustomFormatOption>() {
                Ok(option) => option,
                Err(err) => panic!("unexpected parse error from input \"{}\": {err}", $input),
            };
            assert_eq!(
                parser.peek(),
                None,
                "parser did not reach end of input \"{}\"",
                $input
            );
            assert_eq!(
                option.content, $expected,
                "unexpected content parsed from input \"{}\"",
                $input
            );
        }};
        ($input:literal, Err($error:literal)) => {{
            let src = str_lit! { $input };
            let mut parser = FormatStringParser::new(src.to_slice());
            match parser.parse::<CustomFormatOption>() {
                Ok(option) => panic!(
                    "expected parse error from input \"{}\", got option: {:?}",
                    $input, option.content
                ),
                Err(err) => {
                    let err_msg = err.to_string();
                    assert_eq!(
                        err_msg, $error,
                        "unexpected parse error message from input \"{}\"",
                        $input
                    );
                }
            }
            assert_eq!(
                parser.peek(),
                None,
                "parser did not reach end of input \"{}\"",
                $input
            );
        }};
    }

    #[test]
    fn basic() {
        assert_parse_result! {"[custom_format]", Ok("custom_format")};
        assert_parse_result! {"##[custom_format]##", Ok("custom_format")};
        assert_parse_result! {"###[]###", Ok("")};
        assert_parse_result! {"###[with #] in the option]###", Ok("with #] in the option")};
        assert_parse_result! {"###[with #]# in the option]###", Ok("with #]# in the option")};
        assert_parse_result! {"###[with #]## in the option]###", Ok("with #]## in the option")};

        assert_parse_result! {
            "[missing end",
            Err(r###"Did not find the required ending sequence "]" to end the custom format option:
   --> /inside/a/test.rs:999:124
    |
999 | "[missing end"
    |  ^^^^^^^^^^^^
"###)
        };
        assert_parse_result! {
            "#[missing end",
            Err(r###"Did not find the required ending sequence "]#" to end the custom format option:
   --> /inside/a/test.rs:999:124
    |
999 | "#[missing end"
    |  ^^^^^^^^^^^^^
"###)
        };
        assert_parse_result! {
            "##[missing end",
            Err(r###"Did not find the required ending sequence "]##" to end the custom format option:
   --> /inside/a/test.rs:999:124
    |
999 | "##[missing end"
    |  ^^^^^^^^^^^^^^
"###)
        };
    }

    #[test]
    fn leaves_rest() {
        let src = str_lit! { "###[with ]### in the option]###" };
        let mut parser = FormatStringParser::new(src.to_slice());
        assert!(parser.parse::<CustomFormatOption>().is_ok());
        let mut rest = String::new();
        while let Ok((_, c)) = parser.take() {
            rest.push(c);
        }
        assert_eq!(rest, " in the option]###");
    }

    #[test]
    fn reports_missing_start() {
        let src = str_lit! { "##not starting right" };
        let mut parser = FormatStringParser::new(src.to_slice());
        let Err(err) = parser.parse::<CustomFormatOption>() else {
            panic!("expected parse error from input \"##not starting right\"");
        };
        let err = err.to_string();

        assert_eq!(
            err,
            r###"expected `#` or `[` to start custom format option, found `n`:
   --> /inside/a/test.rs:999:126
    |
999 | "##not starting right"
    |    ^
"###
        );
    }

    #[test]
    fn counts_escapes() {
        let src = str_lit! { "###[custom_format]###" };
        let mut parser = FormatStringParser::new(src.to_slice());
        let option = parser.parse::<CustomFormatOption>().unwrap();
        assert_eq!(option.num_escapes, 3);

        let src = str_lit! { "#[custom_format]#" };
        let mut parser = FormatStringParser::new(src.to_slice());
        let option = parser.parse::<CustomFormatOption>().unwrap();
        assert_eq!(option.num_escapes, 1);

        let src = str_lit! { "[custom_format]" };
        let mut parser = FormatStringParser::new(src.to_slice());
        let option = parser.parse::<CustomFormatOption>().unwrap();
        assert_eq!(option.num_escapes, 0);
    }
}
