use super::*;

use quote::{ToTokens, quote_spanned};

impl<'a> ToTokens for FormatOptions<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
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

impl ToTokens for NumberFormatOption {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use NumberFormatOption::*;
        tokens.extend(quote! { ::sscanf::advanced::NumberFormatOption:: });
        tokens.extend(match self {
            Binary(policy) => quote! { Binary(#policy) },
            Octal(policy) => quote! { Octal(#policy) },
            Decimal => quote! { Decimal },
            Hexadecimal(policy) => quote! { Hexadecimal(#policy) },
            Other(base) => quote! { Other(#base) },
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

impl<'a> ToTokens for CustomFormatOption<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let span = self.src.span();
        let text = &self.custom;
        tokens.extend(quote_spanned! {span=> ::std::cow::Cow::Borrowed(#text)});
    }
}
