use super::*;

pub(crate) fn parse_bracket_content<I: Iterator<Item = (usize, char)>>(
    input: &mut std::iter::Peekable<I>,
    src: &ScanfInner,
    start: usize,
) -> Result<Option<PlaceHolder>> {
    let mut type_name = String::new();
    let mut type_token = None;
    let mut in_type = true;
    let mut config = String::new();
    let mut has_config = false;
    let mut config_start = 0;
    while let Some((i, c)) = input.next() {
        if in_type && (c == ':' || c == '}') && !type_name.is_empty() {
            let s = start + 1;
            let e = i - 1;
            let span = sub_span(src, (s, e));

            // check if it looks like the old format
            if (c == '}' && get_radix(&type_name).is_some()) || (type_name.starts_with('/') && type_name.ends_with('/')) {
                let msg = format!(
                    "It looks like you are using the old format.\nconfig options now require a ':' prefix like so: {{:{}}}",
                    type_name
                );
                return sub_error_result(&msg, src, (s, e));
            }

            let tokens = type_name
                .parse::<TokenStream>()
                .map_err(|err| err.to_string())
                .and_then(|tokens| {
                    syn::parse2::<Path>(quote_spanned!(span => #tokens))
                        .map_err(|err| err.to_string())
                })
                .map_err(|err| {
                    sub_error(
                        &format!("Invalid type in placeholder: {}.\nIf this is supposed to be a format option, prefix it with a ':'", err),
                        src,
                        (s, e),
                    )
                })?;
            type_token = Some(tokens);
        }
        if c == '\\' && !in_type {
            // escape any curly brackets (double \\ are halved to enable the use of \{ and \})
            if let Some((_, next_c)) = input.next_if(|x| x.1 == '{' || x.1 == '}' || x.1 == '\\') {
                config.push(next_c);
            } else {
                config.push('\\');
            }
        } else if c == ':' && in_type {
            in_type = false;
            has_config = true;
            config_start = i + 1;
        } else if c == '{' {
            if i == start + 1 {
                // '{' followed by '{' => escaped '{{'
                return Ok(None);
            }
            return sub_error_result(
                "Expected '}' after given '{', got '{' instead. Curly Brackets inside format options must be escaped with '\\'",
                src,
                (start, i),
            );
        } else if c == '}' {
            return Ok(Some(PlaceHolder {
                name: String::new(),
                type_token,
                config: has_config.then(|| (config, config_start)),
                span: (start, i),
            }));
        } else {
            // anything else is part of the type or config
            if in_type {
                type_name.push(c);
            } else {
                config.push(c);
            }
        }
    }
    // end of input string
    sub_error_result(
        "Missing '}' after given '{'. Literal '{' need to be escaped as '{{'",
        src,
        (start, src.fmt.len()),
    )
}

pub(crate) fn regex_from_config(
    config: &str,
    config_span: (usize, usize),
    ty: &Path,
    src: &ScanfInner,
) -> Result<(TokenStream, Option<TokenStream>)> {
    let ty_string = ty.to_token_stream().to_string();
    if let Some(radix) = get_radix(config) {
        let binary_digits = binary_length(&ty_string).ok_or_else(|| {
            Error::new_spanned(
                ty,
                "radix options only work on primitive numbers from std with no path or type alias",
            )
        })?;
        // digit conversion: digits_base / log_x(base) * log_x(target) with any log-base x,
        // so we choose log_2 where log_2(target) = 1;
        let binaries_per_digit = (radix as f32).log2();
        let digits = (binary_digits as f32 / binaries_per_digit).ceil() as u8;

        // possible characters for digits
        use std::cmp::Ordering::*;
        let mut regex = match radix.cmp(&10) {
            Less => format!(r"[0-{}]", radix - 1),
            Equal => r"[0-9aA]".to_string(),
            Greater => {
                let last_letter = (b'a' + radix - 10) as char;
                format!(r"[0-9a-{}A-{}]", last_letter, last_letter.to_uppercase())
            }
        };
        // repetition factor
        regex += &format!(r"{{1, {}}}", digits);

        // optional prefix
        let prefix = if radix == 16 {
            Some("0x")
        } else if radix == 8 {
            Some("0o")
        } else if radix == 2 {
            Some("0b")
        } else {
            None
        };
        if let Some(pref) = prefix.as_ref() {
            regex = format!(r"{}{1}|{1}", pref, regex);
        }

        let span = ty.span();
        let radix = radix as u32;
        let input_mapper = if let Some(prefix) = prefix {
            quote!(input.strip_prefix(#prefix).unwrap_or(input))
        } else {
            quote!(input)
        };
        let converter = quote_spanned!(span => #ty::from_str_radix(#input_mapper, #radix));

        Ok((quote!(#regex), Some(converter)))
    } else if let Some(regex) = config.strip_prefix('/').and_then(|s| s.strip_suffix('/')) {
        if let Err(err) = regex_syntax::Parser::new().parse(regex) {
            return sub_error_result(
                &format!("{}\n\nIn custom Regex format option", err),
                src,
                config_span,
            );
        }
        Ok((quote!(#regex), None))
    } else {
        let function_name = match ty_string.as_str() {
            "DateTime" | "NaiveDate" | "NaiveTime" | "NaiveDateTime" => quote!(::parse_from_str),
            "Utc" | "Local" => quote!(.datetime_from_str),
            _ => return sub_error_result(
                &format!("Unknown format option: '{}'.\nHint: regex format options must start and end with '/'", config),
                src,
                config_span,
            ),
        };
        let span = ty.span();
        let (regex, chrono_fmt) = chrono::map_chrono_format(config, src, config_span.0)?;
        let converter = wrap_in_feature_gate(
            quote_spanned!(span => ::sscanf::chrono::#ty #function_name(input, #chrono_fmt)),
            ty,
        );
        Ok((quote!(#regex), Some(converter)))
    }
}

fn get_radix(config: &str) -> Option<u8> {
    match config {
        "x" => Some(16),
        "o" => Some(8),
        "b" => Some(2),
        _ => {
            let radix = config.strip_prefix('r')?.parse::<u8>().ok()?;

            // Range taken from: https://doc.rust-lang.org/std/primitive.usize.html#panics
            if (2..=36).contains(&radix) {
                Some(radix)
            } else {
                None
            }
        }
    }
}

fn binary_length(ty: &str) -> Option<usize> {
    match ty {
        "u8" | "i8" => Some(8),
        "u16" | "i16" => Some(16),
        "u32" | "i32" => Some(32),
        "u64" | "i64" => Some(64),
        "u128" | "i128" => Some(128),
        "usize" | "isize" if usize::MAX as u64 == u32::MAX as u64 => Some(32),
        "usize" | "isize" if usize::MAX as u64 == u64::MAX as u64 => Some(64),
        _ => None,
    }
}

fn wrap_in_feature_gate(tokens: TokenStream, span: impl quote::ToTokens) -> TokenStream {
    let error = Error::new_spanned(
        span,
        "sscanf is missing 'chrono' feature to use chrono types",
    )
    .to_compile_error();
    quote!(::sscanf::chrono_check!({#tokens}, {#error}))
}
