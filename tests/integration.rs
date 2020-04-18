use std::ffi::OsString;
use std::str;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

mod common;
use common::from_bytes;
use common::from_vec;
use common::test_bytes;
use common::test_vec;
use common::INVALID_STRING;

const UTF8_STRING: &str = "string";

const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

fn test_utf8_bytes(string: &str) {
    let os_string = OsString::from(string);
    let string = string.as_bytes();
    assert_eq!(Ok(&os_string), from_bytes(string).as_ref());
    assert_eq!(string, os_string.to_bytes().as_ref());
}

fn test_utf8_vec(string: &str) {
    let os_string = OsString::from(string.to_string());
    let string = string.as_bytes();
    assert_eq!(Ok(&os_string), from_vec(string.to_vec()).as_ref());
    assert_eq!(string, os_string.into_vec().as_slice());
}

fn test_string_is_invalid_utf8(string: &[u8]) {
    assert!(str::from_utf8(string).is_err());
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
fn test_wtf8_string_is_invalid_utf8() {
    test_string_is_invalid_utf8(WTF8_STRING);
}

#[test]
fn test_wtf8_bytes() -> Result<(), EncodingError> {
    test_bytes(WTF8_STRING)
}

#[test]
fn test_wtf8_vec() -> Result<(), EncodingError> {
    test_vec(WTF8_STRING)
}
