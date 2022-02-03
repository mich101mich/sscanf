use super::*;

macro_rules! get_next {
    ($iter: ident, $start: expr, $end: expr, $src: ident) => {
        if let Some(next) = $iter.next() {
            next
        } else {
            return sub_error_result(
                "Incomplete chrono format '%'. Literal '%' need to be escaped as '%%'",
                $src,
                ($start, $end),
            );
        }
    };
}

pub(crate) fn map_chrono_format(f: &str, src: &ScanfInner, offset: usize) -> Result<(String, String)> {
    let mut regex = String::new();
    let chrono_fmt = f.replace("\\{", "{").replace("\\}", "}");

    let mut iter = f
        .chars()
        .enumerate()
        .map(|(i, c)| (i + offset, c))
        .peekable();

    while let Some((i, c)) = iter.next() {
        if c != '%' {
            if regex_syntax::is_meta_character(c) {
                regex.push('\\');
            }
            regex.push(c);
            continue;
        }
        let mut next = get_next!(iter, i, i, src);

        let padding = match next.1 {
            '-' => Some(""),
            '0' => Some("0"),
            '_' => Some(" "),
            _ => None,
        };
        if padding.is_some() {
            next = get_next!(iter, i, next.0, src);
        }

        regex += &get_date_fmt(next, padding, src, &mut iter)?;
    }

    Ok((regex, chrono_fmt))
}

fn get_date_fmt(
    letter: (usize, char),
    padding: Option<&'static str>,
    src: &ScanfInner,
    iter: &mut impl Iterator<Item = (usize, char)>,
) -> Result<String> {
    let i = letter.0;
    let pad = |def| padding.unwrap_or(def);
    let pad_to = |def, n| {
        let padding = pad(def);
        let mut fmt = String::from("(");
        for i in 0..n {
            if i != 0 {
                fmt += "|";
            }
            for _ in 0..i {
                fmt += padding;
            }
            fmt += "[1-9]";
            for _ in (i + 1)..n {
                fmt += r"\d";
            }
        }
        fmt + ")"
    };
    Ok(match letter.1 {
        'Y' | 'G' => pad_to("0", 4),
        'C' | 'y' | 'g' => pad_to("0", 2),
        'm' => format!(r"({}\d|1[0-2])", pad("0")),
        'b' | 'h' => r"[a-zA-Z]{3}".to_string(),
        'B' => r"[a-zA-Z]{3,9}".to_string(),
        'd' => format!(r"({}\d|[12]\d|3[01])", pad("0")),
        'e' => format!(r"({}\d|[12]\d|3[01])", pad(" ")),
        'a' => r"[a-zA-Z]{3}".to_string(),
        'A' => r"[a-zA-Z]+".to_string(),
        'w' => "[0-6]".to_string(),
        'u' => "[1-7]".to_string(),
        'U' | 'W' => format!(r"({}\d|[1-4]\d|5[0-3])", pad("0")),
        'V' => format!(r"({}[1-9]|[1-4]\d|5[0-3])", pad("0")),
        'j' => format!(r"({0}{0}[1-9]|{0}\d\d|[1-3][0-5]\d|[1-3]6[0-6])", pad("0")),
        'D' => format!(
            "{}/{}/{}",
            get_date_fmt((i, 'm'), padding, src, iter)?,
            get_date_fmt((i, 'd'), padding, src, iter)?,
            get_date_fmt((i, 'y'), padding, src, iter)?
        ),
        'x' => format!(
            "{}/{}/{}",
            get_date_fmt((i, 'd'), padding, src, iter)?,
            get_date_fmt((i, 'd'), padding, src, iter)?,
            get_date_fmt((i, 'y'), padding, src, iter)?
        ),
        'F' => format!(
            r"{}\-{}\-{}",
            get_date_fmt((i, 'Y'), padding, src, iter)?,
            get_date_fmt((i, 'm'), padding, src, iter)?,
            get_date_fmt((i, 'd'), padding, src, iter)?
        ),
        'v' => format!(
            r"{}\-{}\-{}",
            get_date_fmt((i, 'e'), padding, src, iter)?,
            get_date_fmt((i, 'b'), padding, src, iter)?,
            get_date_fmt((i, 'Y'), padding, src, iter)?
        ),
        'H' => format!(r"({}\d|1\d|2[0-3])", pad("0")),
        'k' => format!(r"({}\d|1\d|2[0-3])", pad(" ")),
        'I' => format!(r"({}[1-9]|1[0-2])", pad("0")),
        'l' => format!(r"({}[1-9]|1[0-2])", pad(" ")),
        'P' => "(am|pm)".to_string(),
        'p' => "(AM|PM)".to_string(),
        'M' => format!(r"({}\d|[1-5]\d)", pad("0")),
        'S' => format!(r"({}\d|[1-5]\d|60)", pad("0")),
        'f' => r"\d{9}".to_string(),
        '.' => {
            let start = i - 1;
            match get_next!(iter, start, i, src) {
                (_, 'f') => r"\.\d{0,9}".to_string(),
                (ni, c @ '1'..='9') => {
                    if get_next!(iter, start, ni, src).1 == 'f' {
                        format!(r"\.\d{{{}}}", c)
                    } else {
                        return sub_error_result("Incomplete %f specifier ('.' can only appear in combination with %f)", src, (start, ni));
                    }
                }
                _ => return sub_error_result("Incomplete %f specifier ('.' can only appear in combination with %f)", src, (start, i)),
            }
        }
        c @ '1'..='9' => {
            let start = i - 1;
            if get_next!(iter, start, i, src).1 == 'f' {
                format!(r"\d{{{}}}", c)
            } else {
                return sub_error_result("Incomplete %f specifier (numbers can only appear in combination with %f)", src, (start, i));
            }
        }
        'R' => format!(
            "{}:{}",
            get_date_fmt((i, 'H'), padding, src, iter)?,
            get_date_fmt((i, 'M'), padding, src, iter)?,
        ),
        'T' | 'X' => format!(
            "{}:{}:{}",
            get_date_fmt((i, 'H'), padding, src, iter)?,
            get_date_fmt((i, 'M'), padding, src, iter)?,
            get_date_fmt((i, 'S'), padding, src, iter)?,
        ),
        'r' => format!(
            "{}:{}:{} {}",
            get_date_fmt((i, 'I'), padding, src, iter)?,
            get_date_fmt((i, 'M'), padding, src, iter)?,
            get_date_fmt((i, 'S'), padding, src, iter)?,
            get_date_fmt((i, 'p'), padding, src, iter)?,
        ),
        'Z' => r"\w+".to_string(),
        'z' => r"\+\d\d\d\d".to_string(),
        c @ ':' | c @ '#' => {
            if get_next!(iter, i - 1, i, src).1 == 'z' {
                if c == ':' {
                    r"\+\d\d:\d\d".to_string()
                } else {
                    r"\+\d\d(\d\d)?".to_string()
                }
            } else {
                return sub_error_result("Incomplete %z specifier (':' can only appear in combination with %z)", src, (i - 1, i));
            }
        }
        'c' => format!(
            "{} {} {} {} {}",
            get_date_fmt((i, 'a'), padding, src, iter)?,
            get_date_fmt((i, 'h'), padding, src, iter)?,
            get_date_fmt((i, 'e'), padding, src, iter)?,
            get_date_fmt((i, 'X'), padding, src, iter)?,
            get_date_fmt((i, 'Y'), padding, src, iter)?,
        ),
        '+' => format!(
            r"{}T{}\.\d{{0,9}}\+\d\d:\d\d",
            get_date_fmt((i, 'F'), padding, src, iter)?,
            get_date_fmt((i, 'T'), padding, src, iter)?,
        ),
        's' => r"\d+".to_string(),
        't' => '\t'.to_string(),
        'n' => '\n'.to_string(),
        '%' => '%'.to_string(),
        x => return sub_error_result(&format!("Unknown chrono format {}. See https://docs.rs/chrono/^0.4/chrono/format/strftime/ for a full list", x), src, (i, i)),
    })
}
