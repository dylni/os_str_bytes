use std::str;

use os_str_bytes::EncodingError;

mod common;
use common::test_bytes;
use common::test_utf8_bytes;
use common::test_utf8_vec;
use common::test_vec;

const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

const UTF8_STRING: &str = "string";

const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

fn test_string_is_invalid_utf8(string: &[u8]) {
    assert!(str::from_utf8(string).is_err());
}

fn test_invalid_result(result: &Result<(), EncodingError>) {
    if cfg!(windows) {
        assert!(result.is_err());
    } else {
        assert_eq!(&Ok(()), result);
    }
}

#[test]
fn test_empty_bytes() {
    test_utf8_bytes("");
}

#[test]
fn test_empty_vec() {
    test_utf8_vec("");
}

#[test]
fn test_nonempty_utf8_bytes() {
    test_utf8_bytes(UTF8_STRING);
}

#[test]
fn test_nonempty_utf8_vec() {
    test_utf8_vec(UTF8_STRING);
}

#[test]
fn test_invalid_string_is_invalid_utf8() {
    test_string_is_invalid_utf8(INVALID_STRING);
}

#[test]
fn test_invalid_bytes() {
    test_invalid_result(&test_bytes(INVALID_STRING));
}

#[test]
fn test_invalid_vec() {
    test_invalid_result(&test_vec(INVALID_STRING));
}

#[test]
fn test_wtf8_string_is_invalid_utf8() {
    test_string_is_invalid_utf8(WTF8_STRING);
}

#[test]
fn test_wtf8_bytes() {
    assert_eq!(Ok(()), test_bytes(WTF8_STRING));
}

#[test]
fn test_wtf8_vec() {
    assert_eq!(Ok(()), test_vec(WTF8_STRING));
}
