#![cfg(feature = "checked_conversions")]

use std::str;

mod common;
use common::Result;
use common::WTF8_STRING;

fn assert_string_is_invalid_utf8(string: &[u8]) {
    assert!(str::from_utf8(string).is_err());
}

fn assert_invalid_result(result: &Result<()>) {
    if cfg!(windows) {
        assert!(result.is_err());
    } else {
        assert_eq!(&Ok(()), result);
    }
}

#[test]
fn test_empty() {
    common::test_utf8_bytes("");
    common::test_utf8_vec("");
}

#[test]
fn test_nonempty_utf8() {
    const UTF8_STRING: &str = "string";

    common::test_utf8_bytes(UTF8_STRING);
    common::test_utf8_vec(UTF8_STRING);
}

#[test]
fn test_invalid() {
    const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";
    assert_string_is_invalid_utf8(INVALID_STRING);

    assert_invalid_result(&common::test_bytes(INVALID_STRING));
    assert_invalid_result(&common::test_vec(INVALID_STRING));
}

#[test]
fn test_wtf8() {
    const HIGH_SURROGATE: &[u8] = b"\xED\xA0\x80";
    const LOW_SURROGATE: &[u8] = b"\xED\xB0\x80";

    for string in [WTF8_STRING, HIGH_SURROGATE, LOW_SURROGATE] {
        assert_string_is_invalid_utf8(string);

        assert_eq!(Ok(()), common::test_bytes(string));
        assert_eq!(Ok(()), common::test_vec(string));
    }
}
