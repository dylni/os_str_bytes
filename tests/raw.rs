#![cfg(feature = "raw_os_str")]

use std::ffi::OsStr;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::RawOsStr;

mod common;
use common::RAW_WTF8_STRING;

fn from_raw_bytes(string: &[u8]) -> Result<&RawOsStr, EncodingError> {
    // SAFETY: The string is validated before conversion.
    OsStr::from_raw_bytes(string)
        .map(|_| unsafe { common::from_raw_bytes_unchecked(string) })
}

#[test]
fn test_ends_with() {
    #[track_caller]
    fn test(result: bool, suffix: &[u8]) {
        let suffix = from_raw_bytes(suffix).unwrap();
        assert_eq!(result, RAW_WTF8_STRING.ends_with_os(suffix));
    }

    test(true, b"");
    test(true, b"r");
    test(true, b"ar");
    test(true, b"bar");
    if cfg!(not(windows)) {
        test(true, b"\xA9bar");
        test(true, b"\x92\xA9bar");
        test(true, b"\x9F\x92\xA9bar");
    }
    test(cfg!(windows), b"\xED\xB2\xA9bar");
    test(true, b"\xF0\x9F\x92\xA9bar");
    test(true, b"\xED\xA0\xBD\xF0\x9F\x92\xA9bar");
    test(true, b"o\xED\xA0\xBD\xF0\x9F\x92\xA9bar");
    test(true, b"oo\xED\xA0\xBD\xF0\x9F\x92\xA9bar");
    test(true, b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar");

    test(false, b"\xED\xA0\xBDbar");
    test(false, b"\xED\xB2\xA9aar");
}

#[test]
fn test_empty_ends_with() {
    #[track_caller]
    fn test(result: bool, suffix: &str) {
        assert_eq!(
            result,
            RawOsStr::from_str("").ends_with_os(RawOsStr::from_str(suffix)),
        );
    }

    test(true, "");
    test(false, "r");
    test(false, "ar");
}

#[test]
fn test_starts_with() {
    #[track_caller]
    fn test(result: bool, prefix: &[u8]) {
        let prefix = from_raw_bytes(prefix).unwrap();
        assert_eq!(result, RAW_WTF8_STRING.starts_with_os(prefix));
    }

    test(true, b"");
    test(true, b"f");
    test(true, b"fo");
    test(true, b"foo");
    test(true, b"foo\xED\xA0\xBD");
    if cfg!(not(windows)) {
        test(true, b"foo\xED\xA0\xBD\xF0");
        test(true, b"foo\xED\xA0\xBD\xF0\x9F");
        test(true, b"foo\xED\xA0\xBD\xF0\x9F\x92");
    }
    test(cfg!(windows), b"foo\xED\xA0\xBD\xED\xA0\xBD");
    test(true, b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9");
    test(true, b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9b");
    test(true, b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9ba");
    test(true, b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar");

    test(false, b"foo\xED\xB2\xA9");
    test(false, b"fof\xED\xA0\xBD\xED\xA0\xBD");
}

#[test]
fn test_empty_starts_with() {
    #[track_caller]
    fn test(result: bool, prefix: &str) {
        assert_eq!(
            result,
            RawOsStr::from_str("").starts_with_os(RawOsStr::from_str(prefix)),
        );
    }

    test(true, "");
    test(false, "f");
    test(false, "fo");
}
