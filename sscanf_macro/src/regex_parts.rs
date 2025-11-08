use crate::*;

/// A pre-packaged TokenStream that will produce a sscanf::advanced::MatchPart
pub struct MatchPart(TokenStream);

impl MatchPart {
    pub fn from_text(text: &str, escape: bool) -> Self {
        let tokens = if escape {
            quote! { ::sscanf::advanced::MatchPart::literal(#text) }
        } else {
            quote! { ::sscanf::advanced::MatchPart::regex(#text) }
        };
        Self(tokens)
    }

    pub fn from_type(ty: &Type<'_>, format_options: &FormatOptions) -> Self {
        // proc_macros don't have any type information, so we cannot check if the type
        // implements the trait, so we wrap it in this verbose <#ty as Trait> code,
        // so that the compiler can check if the trait is implemented, and, most importantly,
        // tell the user if they forgot to implement the trait.
        // The code is split into two parts in case the type consists of more
        // than one token (like `std::vec::Vec`), so that the FullSpan workaround can be
        // applied.
        // Addition: In older rust versions (before 1.70 or so), the compiler underlined the
        // entire `<#ty as Trait>::MEMBER` code, so the spans of the type needed to be fully
        // applied to the entire expression. In newer versions, it only underlines the `#ty`
        // itself, so the type should ideally keep its original spans.
        // Combined solution: apply the span to everything around the `#ty` token, but not to
        // the `#ty` token itself.
        // Final expression: `<#ty as ::sscanf::FromScanf>::REGEX`
        //            start:  ^   ^^^^
        //              end:          ^^^^^^^^^^^^^^^^^^^^^^^^^^^
        //         original:   ^^^
        let span = ty.full_span();
        let mut function = span.apply_start(quote! { < });
        ty.to_tokens(&mut function);
        function.extend(span.apply(quote! { as }, quote! { ::sscanf::FromScanf >::get_matcher }));

        Self(quote! { ::sscanf::advanced::MatchPart::Matcher( #function ( & #format_options ) ) })
    }

    fn from_custom_regex(regex: &str) -> MatchPart {
        Self(
            quote! { ::sscanf::advanced::MatchPart::Matcher( ::sscanf::advanced::Matcher::from_regex(#regex).unwrap() ) },
        )
    }
}

impl ToTokens for MatchPart {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

pub struct Parser(TokenStream);

impl Parser {
    pub fn from_type(index: usize, ty: &Type<'_>, format_options: &FormatOptions) -> Self {
        if let Some(name) = &ty.field_name {
            // effectively just `src.parse_field(name, index, format_options)`
            // However, we need to arrange the tokens so that the type is always underlined with a proper error message
            // when something goes wrong.
            // This code optimizes for two cases:
            // 1. The type does not implement FromScanf. In this case, the `#ty` in `let ret: #ty` will be underlined.
            // 2. The type implements FromScanf, but there is a lifetime mismatch.
            //    If we just called `src.parse_field`, the compiler would underline just the `parse_field` with the
            //    error: "borrowed data escapes outside of associated function" and a hint pointing to the fact that
            //    `src` is being borrowed by its method call etc, which is not very helpful.
            //    By using the `MatchTree::parse_field(self, ...)` call syntax, it will instead underline the entire
            //    call with the error message: "lifetime may not live long enough" and the hint that the lifetime of
            //    the FromScanf implementation would need to outlive the lifetime of the parsed type, which is
            //    exactly the error we want to point out.
            let span = ty.full_span();
            let call = span.apply(
                quote! {::sscanf::advanced::MatchTree::parse_field},
                quote! {(&src, #name, #index, &#format_options)},
            );
            Self(quote! {{
                let ret: #ty = #call?;
                ret
            }})
        } else {
            Self(quote! { src.parse_at::<#ty>(#index, &#format_options)? })
        }
    }
}

impl ToTokens for Parser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

pub struct SequenceMatcher {
    pub match_parts: Vec<MatchPart>,
    pub parsers: Vec<Parser>,
}

impl SequenceMatcher {
    pub fn empty() -> Self {
        Self {
            match_parts: vec![],
            parsers: vec![],
        }
    }

    pub fn new(format: &FormatString, type_sources: &[Type], escape_input: bool) -> Result<Self> {
        let mut ret = Self::empty();

        // if there are n types, there are n+1 regex_parts, so add the first n during this loop and
        // add the last one afterwards
        for (match_index, ((part, ph), ty)) in format
            .parts
            .iter()
            .zip(format.placeholders.iter())
            .zip(type_sources)
            .enumerate()
        {
            if !part.is_empty() {
                ret.match_parts
                    .push(MatchPart::from_text(part, escape_input));
            }

            let match_part = if let Some(custom) = &ph.config.regex {
                MatchPart::from_custom_regex(&custom.regex)
            } else {
                MatchPart::from_type(ty, &ph.config)
            };
            ret.match_parts.push(match_part);

            let parser = Parser::from_type(match_index, ty, &ph.config);
            ret.parsers.push(parser);
        }

        // add the last regex_part
        if let Some(last_part) = format.parts.last()
            && !last_part.is_empty()
        {
            ret.match_parts
                .push(MatchPart::from_text(last_part, escape_input));
        }

        Ok(ret)
    }

    pub fn get_matcher(&self) -> TokenStream {
        let match_parts = &self.match_parts;
        quote! { ::sscanf::advanced::Matcher::from_sequence(vec![ #(#match_parts),* ]) }
    }
}
