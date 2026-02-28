use super::*;

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

impl FromFormatString<'_> for NumberFormatOption {
    /// Parse a number format option from the given parser
    ///
    /// "...{<ident>:...#r16...}..."
    ///                 |   ^parser when done
    ///                 \_parser
    fn parse(parser: &mut FormatStringParser) -> Result<Self> {
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
                    return parser.err_at(pos1, msg);
                };
                let d2 = parser.map_take_if(|c| c.to_digit(10)).map(|(_, d2)| d2);

                let radix = if let Some(d2) = d2 { d1 * 10 + d2 } else { d1 };

                if !(2..=36).contains(&radix) {
                    // Range taken from: https://doc.rust-lang.org/std/primitive.usize.html#panics
                    let msg = "radix has to be a number between 2 and 36";
                    return parser.err_since(start, msg);
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
                // technically unreachable, since this is already checked before calling this function
                if let Some(hashtag_pos) = hashtag_pos {
                    // The hashtag might have belonged to the previous format option
                    let msg = "This hashtag was interpreted as the start of a number format option, which has to be followed by 'b', 'o', 'x' or 'r<n>'";
                    return parser.err_at(hashtag_pos, msg);
                }
                let msg = "number format option has to start with 'b', 'o', 'x' or 'r<n>'";
                return parser.err_at(pos1, msg);
            }
        };

        Ok(kind)
    }
}

impl ToTokens for NumberFormatOption {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use NumberFormatOption::*;
        tokens.extend(quote! { ::sscanf::advanced::NumberFormatOption:: });
        tokens.extend(match self {
            Binary(policy) => quote! { Binary(#policy) },
            Octal(policy) => quote! { Octal(#policy) },
            Decimal => quote! { Decimal },
            Hexadecimal(policy) => quote! { Hexadecimal(#policy) },
            Other(base) => quote! { Other(::sscanf::advanced::CustomRadix::new(#base).unwrap()) }, // unwrap: we checked the base when parsing
        });
    }
}

impl ToTokens for NumberPrefixPolicy {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use NumberPrefixPolicy::*;
        tokens.extend(quote! { ::sscanf::advanced::NumberPrefixPolicy:: });
        tokens.extend(match self {
            Forbidden => quote! { Forbidden },
            Optional => quote! { Optional },
            Required => quote! { Required },
        });
    }
}
