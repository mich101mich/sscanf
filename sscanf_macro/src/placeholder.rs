use crate::*;

/// A placeholder in a format string
///
/// ```text
/// ...{foo:bar}...
///    ^^^^^^^^^     src
///     ^^^          ident
///         ^^^      config
/// ```
pub struct Placeholder<'a> {
    pub src: StrLitSlice<'a>,
    pub ident: Option<StrLitSlice<'a>>,
    pub config: Option<FormatOption<'a>>,
}

impl<'a> Placeholder<'a> {
    pub fn new<I: Iterator<Item = (usize, char)>>(
        input: &'_ mut std::iter::Peekable<I>,
        src: &'_ StrLitSlice<'a>,
        start: usize,
    ) -> Result<Self> {
        let mut ident_start = None;
        let mut ident = None;
        let mut has_colon = false;
        let mut end = None;
        while let Some((i, c)) = input.next() {
            if c == '}' {
                if let Some(ident_start) = ident_start {
                    ident = Some(src.slice(ident_start..i));
                }
                end = Some(i);
                break;
            } else if c == ':' && input.next_if(|(_, c)| *c == ':').is_none() {
                // a single ':'
                has_colon = true;
                if let Some(ident_start) = ident_start {
                    ident = Some(src.slice(ident_start..i));
                }
                break;
            } else if ident_start.is_none() {
                ident_start = Some(i);
            }
        }
        let mut config = None;
        if has_colon {
            let (cfg, end_i) = FormatOption::new(input, src, start)?;
            config = Some(cfg);
            end = Some(end_i);
        } else if let Some(ident) = ident.as_ref() {
            if ident.text().starts_with('/') && ident.text().ends_with('/') {
                // types/fields cannot start with a slash
                let msg = format!(
                    "missing `:` in front of custom regex. Write `{{:{}}}` instead",
                    ident.text()
                );
                return ident.err(&msg); // checked in tests/fail/<channel>/invalid_placeholder.rs
            }
        }

        let end = end.ok_or_else(|| src.slice(start..).error(MISSING_CLOSE_STRING))?; // checked in tests/fail/<channel>/invalid_placeholder.rs

        let src = src.slice(start..=end);

        Ok(Placeholder { src, ident, config })
    }
}
