#![cfg(feature = "raw_os_str")]

use std::ffi::OsStr;

use os_str_bytes::RawOsStr;

#[macro_use]
mod raw_common;

if_conversions! {
    use os_str_bytes::OsStrBytes;

    use raw_common::RAW_WTF8_STRING;
}

fn test(result: &[(&OsStr, &str)], string: &RawOsStr) {
    assert_eq!(
        result,
        string
            .utf8_chunks()
            .map(|(invalid, valid)| (invalid.as_os_str(), valid))
            .collect::<Vec<_>>(),
    );
}

#[test]
fn test_empty() {
    test(&[], RawOsStr::new(""));
}

if_conversions! {
    #[test]
    fn test_wft8() {
        test(
            &[
                (OsStr::new(""), "foo"),
                (
                    &OsStr::assert_from_raw_bytes(&b"\xED\xA0\xBD"[..]),
                    "\u{1F4A9}bar",
                ),
            ],
            &RAW_WTF8_STRING,
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_invalid() {
        test(
            &[
                (OsStr::new(""), "foo"),
                (&OsStr::assert_from_raw_bytes(&b"\xF1\xF1\x80"[..]), "bar"),
                (&OsStr::assert_from_raw_bytes(&b"\xF1\x80\x80"[..]), ""),
            ],
            &RawOsStr::assert_cow_from_raw_bytes(
                b"foo\xF1\xF1\x80bar\xF1\x80\x80",
            ),
        );
    }
}
