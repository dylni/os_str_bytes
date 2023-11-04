#![cfg(feature = "raw_os_str")]

use os_str_bytes::RawOsStr;

#[macro_use]
mod raw_common;

if_conversions! {
    use raw_common::RAW_WTF8_STRING;
}

fn test(result: &str, string: &RawOsStr) {
    assert_eq!(format!("RawOsStr({})", result), format!("{:?}", string));
    assert_eq!(
        format!("RawOsString({})", result),
        format!("{:?}", string.to_owned()),
    );
}

#[test]
fn test_empty() {
    test("\"\"", RawOsStr::new(""));
}

if_conversions! {
    #[test]
    fn test_wft8() {
        let wchar = if cfg!(unix) {
            "\\xED\\xA0\\xBD"
        } else {
            "\\u{D83D}"
        };
        test(&format!("\"foo{}\u{1F4A9}bar\"", wchar), &RAW_WTF8_STRING);
    }
}

#[test]
fn test_quote() {
    test("\"foo\\\"bar\"", RawOsStr::new("foo\"bar"));
}
