use crate::*;

use quote::{ToTokens, quote_spanned};

mod custom;
mod number;
mod regex;
pub use custom::*;
pub use number::*;
pub use regex::*;

/// Replica of `sscanf::advanced::FormatOptions`, but with an additional `regex` field
#[derive(Clone)]
pub struct FormatOptions<'a> {
    pub src: StrLitSlice<'a>,
    pub regex: Option<RegexOverride<'a>>,
    pub number: Option<NumberFormatOption>,
    pub custom: Option<CustomFormatOption<'a>>,
}

impl ErrorTarget for FormatOptions<'_> {
    fn error(&self, message: impl Display) -> Error {
        self.src.error(message)
    }
}

// the most recent option that was parsed, used for error messages
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
}

impl<'a> FromFormatString<'a> for FormatOptions<'a> {
    /// Parse format options from the given parser
    ///
    /// "...{<ident>:<config>}..."
    ///              ^parser  ^parser when done
    fn parse(parser: &mut FormatStringParser<'a>) -> Result<Self> {
        let mut ret = FormatOptions::empty(parser.slice_since(parser.get_open_bracket_pos()));

        let mut most_recent = OneOption::None; // the most recent option that was parsed, used for error messages

        loop {
            let (start, c) = parser.peek_required()?;
            if c == '}' {
                parser.take()?;
                break;
            } else if c == ' ' {
                // whitespace is allowed between options
                parser.take()?;
                most_recent = OneOption::None; // most recent is no longer directly adjacent to the next option
            } else if c == '/' {
                // regex option
                if ret.regex.is_some() {
                    let msg = "multiple regex options are not allowed";
                    return parser.err_at(start, msg);
                }
                ret.regex = Some(parser.parse()?);
                most_recent = OneOption::Regex;
            } else if c == '[' || (c == '#' && matches!(parser.peek2(), Some((_, '#' | '[')))) {
                // custom format option
                if ret.custom.is_some() {
                    let msg = "multiple custom format options are not allowed";
                    return parser.err_at(start, msg);
                }
                let custom: CustomFormatOption = parser.parse()?;
                most_recent = OneOption::Custom(custom.clone());
                ret.custom = Some(custom);
            } else if matches!(c, 'b' | 'o' | 'x' | 'r')
                || (c == '#' && matches!(parser.peek2(), Some((_, 'b' | 'o' | 'x' | 'r'))))
            {
                // number format option
                let new_number: NumberFormatOption = parser.parse()?; // parse first to see if our assumption is correct
                if ret.number.is_some() {
                    let msg = "multiple number format options are not allowed";
                    return parser.err_at(start, msg);
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
                return parser.err_at(start, msg);
            }
        }
        Ok(ret)
    }
}

impl ToTokens for FormatOptions<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut modifiers = TokenStream::new();

        if let Some(number) = &self.number {
            modifiers.extend(quote! { options.number = #number; });
        }

        if let Some(custom) = &self.custom {
            modifiers.extend(quote! { options.custom = #custom; });
        }

        let span = self.src.span();
        if modifiers.is_empty() {
            tokens.extend(quote_spanned! {span=> ::sscanf::advanced::FormatOptions::default() });
        } else {
            tokens.extend(quote_spanned! {span=> {
                let mut options = ::sscanf::advanced::FormatOptions::default();
                #modifiers
                options
            }});
        }
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
                parser.err_at(hashtag_pos, msg)
            }
            OneOption::Regex => {
                // {/regex/#}
                let msg = "hashtag '#' has to be followed by a number format option or a custom format option";
                parser.err_at(hashtag_pos, msg)
            }
            OneOption::Custom(custom_format_option) => {
                if custom_format_option.num_escapes > 0 {
                    // {#custom##}
                    let msg = "Unbalanced hashtags '#' around custom format option";
                    parser.err_at(hashtag_pos, msg)
                } else {
                    // {custom#}
                    let msg = "unexpected hashtag '#' after custom format option.
If you meant to add an escape, add another hashtag before the '['. If you meant to start a number format option, continue typing";
                    parser.err_at(hashtag_pos, msg)
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
                    parser.err_at(hashtag_pos, msg)
                } else if matches!(
                    number_format_option,
                    Binary(Optional) | Octal(Optional) | Hexadecimal(Optional)
                ) {
                    // {b#}, {o#}, {x#}
                    let msg = "hashtag '#' has to be placed before the number format option, not after it";
                    parser.err_at(hashtag_pos, msg)
                } else {
                    // {r<n>#}
                    let msg = "unexpected hashtag '#' after number format option";
                    parser.err_at(hashtag_pos, msg)
                }
            }
        }
    } else {
        // '#' followed by something else
        let msg = format!(
        "unexpected '{next}' after hashtag '#'.
Hashtag '#' has to be followed by 'b', 'o', 'x' for a number format option, or '#' or '[' for a custom format option",
    );
        parser.err_at(next_pos, msg)
    }
}
