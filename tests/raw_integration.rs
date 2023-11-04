#![cfg(feature = "raw_os_str")]

use os_str_bytes::RawOsStr;

#[macro_use]
mod raw_common;

if_conversions! {
    use raw_common::RAW_WTF8_STRING;
}

if_conversions! {
    #[test]
    fn test_ends_with() {
        #[track_caller]
        fn test(result: bool, suffix: &[u8]) {
            let suffix = RawOsStr::assert_cow_from_raw_bytes(suffix);
            assert_eq!(result, RAW_WTF8_STRING.ends_with_os(&suffix));
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
                RawOsStr::from_str("")
                    .ends_with_os(RawOsStr::from_str(suffix)),
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
            let prefix = RawOsStr::assert_cow_from_raw_bytes(prefix);
            assert_eq!(result, RAW_WTF8_STRING.starts_with_os(&prefix));
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
                RawOsStr::from_str("")
                    .starts_with_os(RawOsStr::from_str(prefix)),
            );
        }

        test(true, "");
        test(false, "f");
        test(false, "fo");
    }
}

if_conversions! {
    #[should_panic = "cannot split using an empty pattern"]
    #[test]
    fn test_split_by_empty() {
        let _ = RAW_WTF8_STRING.split("");
    }
}

#[should_panic = "cannot split using an empty pattern"]
#[test]
fn test_split_empty_by_empty() {
    let _ = RawOsStr::from_str("").split("");
}
