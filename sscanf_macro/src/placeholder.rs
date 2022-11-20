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
        expect_lowercase_ident: bool,
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
            } else if c == ':' && !input.next_if(|(_, c)| *c == ':').is_some() {
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
        } else if !expect_lowercase_ident {
            // check if ident looks like the old format
            if let Some(ident) = ident.as_ref() {
                if ident.text().starts_with('/')
                    || ident
                        .text()
                        .strip_prefix('r')
                        .and_then(|s| s.parse::<usize>().ok())
                        .is_some()
                    || matches!(ident.text(), "x" | "o" | "b")
                {
                    let msg = format!(
                        "It looks like you are using the old format.
config options now require a ':' prefix like so: {{:{}}}.
If this is actually a type whose name happens to match the old format, sorry.
Please create an UpperCamelCased wrapper type for it.",
                        ident.text()
                    );
                    return ident.err(&msg); // checked in tests/fail/<channel>/old_formats.rs
                }
            }
        }

        let end = end.ok_or_else(|| src.slice(start..).error(MISSING_CLOSE_STRING))?;

        let src = src.slice(start..=end);

        Ok(Placeholder { src, ident, config })
    }
}
