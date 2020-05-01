#![cfg(feature = "raw")]

use os_str_bytes::raw;

mod common;
use common::WTF8_STRING;

#[test]
fn test_ends_with() {
    test(true, b"");
    test(true, b"r");
    test(true, b"ar");
    test(true, b"bar");
    #[cfg(not(windows))]
    {
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

    assert_eq!(true, raw::ends_with("", b""));
    assert_eq!(false, raw::ends_with("", b"r"));
    assert_eq!(false, raw::ends_with("", b"ar"));

    fn test(result: bool, suffix: &[u8]) {
        assert_eq!(result, raw::ends_with(WTF8_STRING, suffix));
    }
}

#[test]
fn test_starts_with() {
    test(true, b"");
    test(true, b"f");
    test(true, b"fo");
    test(true, b"foo");
    test(true, b"foo\xED\xA0\xBD");
    #[cfg(not(windows))]
    {
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

    assert_eq!(true, raw::starts_with("", b""));
    assert_eq!(false, raw::starts_with("", b"f"));
    assert_eq!(false, raw::starts_with("", b"fo"));

    fn test(result: bool, prefix: &[u8]) {
        assert_eq!(result, raw::starts_with(WTF8_STRING, prefix));
    }
}
