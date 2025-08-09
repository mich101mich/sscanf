use super::*;

/// the most recent option that was parsed, used for error messages
enum OneOption<'a> {
    None,
    Regex,
    Custom(CustomFormatOption<'a>),
    Number(NumberFormatOption),
}

impl<'a> FormatOptions<'a> {
    pub fn empty(src: StrLitSlice<'a>) -> Self {
        Self {
            src,
            regex: None,
            number: None,
            custom: None,
        }
    }

    /// Parse format options from the given parser
    ///
    /// "...{<ident>:<config>}..."
    ///              ^parser  ^parser when done
    pub fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let mut ret = FormatOptions::empty(parser.slice_since(parser.get_open_bracket_pos()));

        let mut most_recent = OneOption::None; // the most recent option that was parsed, used for error messages

        loop {
            let (start, c) = parser.peek_required()?;
            if c == '}' {
                parser.take()?;
                break;
            } else if c == '/' {
                // regex option
                if ret.regex.is_some() {
                    let msg = "multiple regex options are not allowed";
                    return parser.err_at(start, msg); // TODO: check
                }
                ret.regex = Some(RegexOverride::parse(parser)?);
                most_recent = OneOption::Regex;
            } else if c == '[' || (c == '#' && matches!(parser.peek2(), Some((_, '#' | '[')))) {
                // custom format option
                if ret.custom.is_some() {
                    let msg = "multiple custom format options are not allowed";
                    return parser.err_at(start, msg); // TODO: check
                }
                let custom = CustomFormatOption::parse(parser)?;
                most_recent = OneOption::Custom(custom.clone());
                ret.custom = Some(custom);
            } else if matches!(c, 'b' | 'o' | 'x' | 'r')
                || (c == '#' && matches!(parser.peek2(), Some((_, 'b' | 'o' | 'x' | 'r'))))
            {
                // number format option
                let new_number = NumberFormatOption::parse(parser)?; // parse first to see if our assumption is correct
                if ret.number.is_some() {
                    let msg = "multiple number format options are not allowed";
                    return parser.err_at(start, msg); // TODO: check
                }
                most_recent = OneOption::Number(new_number);
                ret.number = Some(new_number);
            } else {
                // unknown format option
                if c == '#' {
                    report_unexpected_hashtag(parser, &most_recent, start)?;
                }
                let msg = format!(
                    "unknown format option starting with '{c}'.
Use 'b', 'o', 'x' or 'r' for number format options, '/' for regex, or '[' for custom format options"
                );
                return parser.err_at(start, msg); // TODO: check
            }
        }
        Ok(ret)
    }
}

impl<'a> NumberFormatOption {
    /// Parse a number format option from the given parser
    ///
    /// "...{<ident>:...#r16...}..."
    ///                 |   ^parser when done
    ///                 \_parser
    pub fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let start = parser.get_pos();

        let hashtag_pos = parser.take_if_eq('#').map(|(pos, _)| pos);
        let base_prefix_policy = if hashtag_pos.is_some() {
            // if there is a hashtag, the prefix is always required
            NumberPrefixPolicy::Required
        } else {
            // otherwise, the prefix is optional
            NumberPrefixPolicy::Optional
        };

        let (pos1, c1) = parser.take()?;

        let kind = match c1 {
            'x' => NumberFormatOption::Hexadecimal(base_prefix_policy),
            'o' => NumberFormatOption::Octal(base_prefix_policy),
            'b' => NumberFormatOption::Binary(base_prefix_policy),
            'r' => {
                let (_pos2, d1) = parser.take()?;
                let Some(d1) = d1.to_digit(10) else {
                    let msg = "radix option 'r' has to be followed by a number";
                    return parser.err_at(pos1, msg); // TODO: check
                };
                let d2 = parser.map_take_if(|c| c.to_digit(10)).map(|(_, d2)| d2);

                let radix = if let Some(d2) = d2 { d1 * 10 + d2 } else { d1 };

                if !(2..=36).contains(&radix) {
                    // Range taken from: https://doc.rust-lang.org/std/primitive.usize.html#panics
                    let msg = "radix has to be a number between 2 and 36";
                    return parser.err_since(start, msg); // TODO: check
                }

                if let Some(hashtag_pos) = hashtag_pos {
                    return parser.err_at(
                        hashtag_pos,
                        "radix option 'r' cannot be used with a hashtag since it can't have a prefix",
                    );
                }

                match radix {
                    2 => NumberFormatOption::Binary(NumberPrefixPolicy::Forbidden),
                    8 => NumberFormatOption::Octal(NumberPrefixPolicy::Forbidden),
                    10 => NumberFormatOption::Decimal,
                    16 => NumberFormatOption::Hexadecimal(NumberPrefixPolicy::Forbidden),
                    _ => NumberFormatOption::Other(radix),
                }
            }
            _ => {
                if let Some(hashtag_pos) = hashtag_pos {
                    // The hashtag might have belonged to the previous format option
                    return parser.err_at(
                        hashtag_pos,
                        "This hashtag was interpreted as the start of a number format option, which has to be followed by 'b', 'o', 'x' or 'r<n>'",
                    );
                } else {
                    return parser.err_at(
                        pos1,
                        "number format option has to start with 'b', 'o', 'x' or 'r<n>'",
                    );
                }
            }
        };

        Ok(kind)
    }
}

impl<'a> RegexOverride<'a> {
    /// "...{<ident>:.../<regex>/...}..."
    ///                 ^parser  ^parser when done
    pub(crate) fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let (start, start_slash) = parser.take()?;
        assert_eq!(start_slash, '/');

        let mut regex = String::new();
        let mut escape = None; // index of the last '\', if any
        loop {
            let (i, c) = parser.take()?;
            if c == '/' {
                if escape.take().is_some() {
                    regex.push('/');
                } else {
                    break;
                }
            } else if c == '\\' {
                // TODO: check/fix escaping logic (and add tests)
                // if !src.is_raw() {
                //     let (_, next) = input
                //         .next()
                //         .ok_or_else(|| src.slice(i..).error("unexpected end of regex"))?;
                //     // the above error is probably not possible, since a single \ at
                //     // the end of a non-raw string would escape the closing " and the
                //     // compiler would already complain about that.
                //     // the check is still here just in case

                //     if next != '\\' {
                //         // regular escaped char (\n, \t, etc)
                //         if escape.take().is_some() {
                //             regex.push('\\');
                //         }
                //         regex.push('\\');
                //         regex.push(next);
                //         continue;
                //     }
                // }
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

impl<'a> CustomFormatOption<'a> {
    /// Parse a custom format option from the given parser
    ///
    /// "...{<ident>:...##[<custom>]##...}..."
    ///                 ^parser       ^parser when done
    pub fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        #![allow(unused, reason = "TODO: used once other todos are done")]
        todo!()
    }
}

fn report_unexpected_hashtag<'a>(
    parser: &mut FormatStringParser<'a>,
    most_recent: &OneOption<'a>,
    hashtag_pos: usize,
) -> Result<()> {
    parser.take()?;
    let (next_pos, next) = parser.peek_required()?;
    if next == '}' {
        // '#' followed by the end of the format string
        match most_recent {
            OneOption::None => {
                // just `{#}`
                let msg = "hashtag '#' has to be followed by 'b', 'o', 'x' for a number format option, or '#' or '[' for a custom format option";
                parser.err_at(hashtag_pos, msg) // TODO: check
            }
            OneOption::Regex => {
                // {/regex/#}
                let msg = "hashtag '#' has to be followed by a number format option or a custom format option";
                parser.err_at(hashtag_pos, msg) // TODO: check
            }
            OneOption::Custom(custom_format_option) => {
                if custom_format_option.num_escapes > 0 {
                    // {#custom##}
                    let msg = "Unbalanced hashtags '#' around custom format option";
                    parser.err_at(hashtag_pos, msg) // TODO: check
                } else {
                    // {custom#}
                    let msg = "unexpected hashtag '#' after custom format option.
If you meant to add an escape, add another hashtag before the '['. If you meant to start a number format option, continue typing";
                    parser.err_at(hashtag_pos, msg) // TODO: check
                }
            }
            OneOption::Number(number_format_option) => {
                use NumberFormatOption::*;
                use NumberPrefixPolicy::*;
                if matches!(
                    number_format_option,
                    Binary(Required) | Octal(Required) | Hexadecimal(Required)
                ) {
                    // {#b#}, {#o#}, {#x#}
                    let msg = "unexpected hashtag '#' after number format option";
                    parser.err_at(hashtag_pos, msg) // TODO: check
                } else if matches!(
                    number_format_option,
                    Binary(Optional) | Octal(Optional) | Hexadecimal(Optional)
                ) {
                    // {b#}, {o#}, {x#}
                    let msg = "hashtag '#' has to be placed before the number format option, not after it";
                    parser.err_at(hashtag_pos, msg) // TODO: check
                } else {
                    // {r<n>#}
                    let msg = "hashtag '#' can't be used with a radix number format option";
                    parser.err_at(hashtag_pos, msg) // TODO: check
                }
            }
        }
    } else {
        // '#' followed by something else
        let msg = format!(
        "unexpected '{next}' after hashtag '#'.
Hashtag '#' has to be followed by 'b', 'o', 'x' for a number format option, or '#' or '[' for a custom format option",
    );
        parser.err_at(next_pos, msg) // TODO: check
    }
}
