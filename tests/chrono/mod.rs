use ::chrono::*;
use sscanf::*;

#[test]
fn date_time() {
    let expected = FixedOffset::east(4 * 3600 + 3600 / 2)
        .ymd(2021, 6, 21)
        .and_hms(13, 37, 42);

    let input = "2021-06-21T13:37:42+04:30";
    let parsed = scanf!(input, "{DateTime<FixedOffset>}");
    assert_eq!(parsed, Ok(expected));

    let parsed = scanf!(input, "{DateTime<Utc>}");
    assert_eq!(parsed, Ok(expected.into()));

    let parsed = scanf!(input, "{DateTime:%Y-%m-%dT%H:%M:%S%:z}");
    assert_eq!(parsed, Ok(expected));

    let parsed = scanf!(input, "{DateTime:%FT%T%:z}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn naive_date() {
    let expected = NaiveDate::from_ymd(2021, 6, 21);
    let input = "2021-06-21";

    let parsed = scanf!(input, "{NaiveDate:%F}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn naive_time() {
    let expected = NaiveTime::from_hms(13, 37, 42);
    let input = "13:37:42";

    let parsed = scanf!(input, "{NaiveTime:%T}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn naive_date_time() {
    let expected = NaiveDate::from_ymd(2021, 6, 21).and_hms(13, 37, 42);
    let input = "2021-06-21 13:37:42";

    let parsed = scanf!(input, "{NaiveDateTime:%Y-%m-%d %H:%M:%S}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn utc() {
    let expected = Utc.ymd(2021, 6, 21).and_hms(13, 37, 42);

    let input = "2021-06-21 13:37:42";
    let parsed = scanf!(input, "{Utc:%Y-%m-%d %H:%M:%S}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn local() {
    let expected = Local.ymd(2021, 6, 21).and_hms(13, 37, 42);

    let input = "2021-06-21 13:37:42";
    let parsed = scanf!(input, "{Local:%Y-%m-%d %H:%M:%S}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn escaping() {
    let expected = Utc.ymd(2021, 6, 21).and_hms(13, 37, 42);

    let input = "{}2021-06-21{} 13:37:42}}";
    let parsed = scanf!(input, r"{{{Utc:\}%Y-%m-%d\{\} %H:%M:%S\}}}}");
    assert_eq!(parsed, Ok(expected));
}

#[test]
fn formats() {
    let expected = Utc.ymd(2021, 6, 1).and_hms(1, 2, 3);

    let input = "2021 June  1 01: 2:3";
    let parsed = scanf!(input, "{Utc:%C%y %B %e %0H:%_M:%-S}");
    assert_eq!(parsed, Ok(expected));

    macro_rules! cmp {
        ($a: literal, $b: literal) => {
            assert_eq!(
                scanf_get_regex!($a, Utc).as_str(),
                scanf_get_regex!($b, Utc).as_str(),
            )
        };
    }

    cmp!("{:%e}", "{:%_d}");
    cmp!("{:%0e}", "{:%d}");
    cmp!("{:%b}", "{:%h}");
    cmp!("{:%U}", "{:%W}");
    cmp!("{:%G}", "{:%Y}");
    cmp!("{:%g}", "{:%y}");

    cmp!("{:%D}", "{:%m/%d/%y}");
    cmp!("{:%x}", "{:%d/%d/%y}");
    cmp!("{:%F}", "{:%Y-%m-%d}");
    cmp!("{:%v}", "{:%e-%b-%Y}");

    cmp!("{:%k}", "{:%_H}");
    cmp!("{:%0k}", "{:%H}");
    cmp!("{:%l}", "{:%_I}");
    cmp!("{:%0l}", "{:%I}");

    cmp!("{:%R}", "{:%H:%M}");
    cmp!("{:%T}", "{:%H:%M:%S}");
    cmp!("{:%X}", "{:%H:%M:%S}");
    cmp!("{:%r}", "{:%I:%M:%S %p}");
    cmp!("{:%c}", "{:%a %b %e %T %Y}");
    cmp!("{:%+}", "{:%FT%T%.f%:z}");
}
