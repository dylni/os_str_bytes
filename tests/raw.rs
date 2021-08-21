#![cfg(feature = "raw_os_str")]

use std::ffi::OsStr;
use std::mem;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::RawOsStr;

mod common;
use common::WTF8_STRING;

unsafe fn from_bytes_unchecked(string: &[u8]) -> &RawOsStr {
    // SAFETY: This implementation detail can only be assumed by this crate.
    mem::transmute(string)
}

fn from_bytes(string: &[u8]) -> Result<&RawOsStr, EncodingError> {
    OsStr::from_raw_bytes(string)
        .map(|_| unsafe { from_bytes_unchecked(string) })
}

#[test]
fn test_ends_with() {
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

    fn test(result: bool, suffix: &[u8]) {
        let wtf8_string = unsafe { from_bytes_unchecked(WTF8_STRING) };
        let suffix = from_bytes(suffix).unwrap();
        assert_eq!(result, wtf8_string.ends_with_os(suffix));
    }
}

#[test]
fn test_empty_ends_with() {
    macro_rules! test {
        ( $result:expr , $string:expr , $substring:expr ) => {
            #[allow(clippy::bool_assert_comparison)]
            {
                assert_eq!(
                    $result,
                    RawOsStr::from_str($string)
                        .ends_with_os(RawOsStr::from_str($substring)),
                );
            }
        };
    }
    test!(true, "", "");
    test!(false, "", "r");
    test!(false, "", "ar");
}

#[test]
fn test_starts_with() {
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

    fn test(result: bool, prefix: &[u8]) {
        let wtf8_string = unsafe { from_bytes_unchecked(WTF8_STRING) };
        let prefix = from_bytes(prefix).unwrap();
        assert_eq!(result, wtf8_string.starts_with_os(prefix));
    }
}

#[test]
fn test_empty_starts_with() {
    macro_rules! test {
        ( $result:expr , $string:expr , $substring:expr ) => {
            #[allow(clippy::bool_assert_comparison)]
            {
                assert_eq!(
                    $result,
                    RawOsStr::from_str($string)
                        .starts_with_os(RawOsStr::from_str($substring)),
                );
            }
        };
    }
    test!(true, "", "");
    test!(false, "", "f");
    test!(false, "", "fo");
}
