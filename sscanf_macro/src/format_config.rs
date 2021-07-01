use super::*;

pub(crate) fn parse_bracket_content<I: Iterator<Item = (usize, char)>>(
    input: &mut std::iter::Peekable<I>,
    src: &SscanfInner,
    start: usize,
) -> Result<Option<PlaceHolder>> {
    let mut config = String::new();
    while let Some((i, c)) = input.next() {
        if c == '\\' {
            // escape any curly brackets
            if let Some((_, c)) = input.next_if(|x| x.1 == '{' || x.1 == '}') {
                config.push(c);
                continue;
            }
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
                config: if i == start + 1 { None } else { Some(config) },
                span: (start, i),
            }));
        } else {
            // anything else is part of the config
            config.push(c);
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
    ty: &TypePath,
    ph: &PlaceHolder,
    src: &SscanfInner,
) -> Result<(TokenStream, TokenStream)> {
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
                let last_letter_upper = (b'A' + radix - 10) as char;
                format!(r"[0-9a-{}A-{}]", last_letter, last_letter_upper)
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
        let name = &ph.name;
        let radix = radix as u32;
        let matcher = if let Some(prefix) = prefix {
            quote_spanned!(span => #ty::from_str_radix({
                let s = cap.name(#name)?.as_str();
                s.strip_prefix(#prefix).unwrap_or(s)
            }, #radix).ok()?)
        } else {
            quote_spanned!(span => #ty::from_str_radix(cap.name(#name)?.as_str(), #radix).ok()?)
        };

        Ok((quote!(#regex), matcher))
    } else {
        match ty_string.as_str() {
            "DateTime" | "NaiveDate" | "NaiveTime" | "NaiveDateTime" => {
                let (regex, chrono_fmt) = chrono::map_chrono_format(config, src, ph.span.0)?;

                let span = ty.span();
                let name = &ph.name;
                let matcher = quote_spanned!(span =>
                    ::sscanf::chrono::#ty::parse_from_str(cap.name(#name)?.as_str(), #chrono_fmt)
                        .expect("chrono parsing should be exact")
                );

                Ok((quote!(#regex), matcher))
            }
            "Utc" | "Local" => {
                let (regex, chrono_fmt) = chrono::map_chrono_format(config, src, ph.span.0)?;

                let span = ty.span();
                let name = &ph.name;
                let matcher = quote_spanned!(span =>
                    #ty.datetime_from_str(cap.name(#name)?.as_str(), #chrono_fmt).ok()?
                );

                Ok((quote!(#regex), matcher))
            }
            _ => sub_error_result(
                &format!("Unknown format option: '{}'", config),
                src,
                ph.span,
            ),
        }
    }
}

fn get_radix(config: &str) -> Option<u8> {
    if let Some(n) = config
        .strip_prefix('r')
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|n| *n >= 2 && *n <= 36)
    {
        return Some(n);
    }
    match config {
        "x" => Some(16),
        "o" => Some(8),
        "b" => Some(2),
        _ => None,
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
