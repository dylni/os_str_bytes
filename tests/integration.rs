use std::str;

#[macro_use]
mod common;
use common::Result;
use common::WTF8_STRING;

if_raw_str! {
    use os_str_bytes::RawOsStr;

    use common::RAW_WTF8_STRING;
}

const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

const UTF8_STRING: &str = "string";

fn test_string_is_invalid_utf8(string: &[u8]) {
    assert!(str::from_utf8(string).is_err());
}

fn test_invalid_result(result: &Result<()>) {
    if cfg!(windows) {
        assert!(result.is_err());
    } else {
        assert_eq!(&Ok(()), result);
    }
}

#[test]
fn test_empty_bytes() {
    common::test_utf8_bytes("");
}

#[test]
fn test_empty_vec() {
    common::test_utf8_vec("");
}

#[test]
fn test_nonempty_utf8_bytes() {
    common::test_utf8_bytes(UTF8_STRING);
}

#[test]
fn test_nonempty_utf8_vec() {
    common::test_utf8_vec(UTF8_STRING);
}

#[test]
fn test_invalid_string_is_invalid_utf8() {
    test_string_is_invalid_utf8(INVALID_STRING);
}

#[test]
fn test_invalid_bytes() {
    test_invalid_result(&common::test_bytes(INVALID_STRING));
}

#[test]
fn test_invalid_vec() {
    test_invalid_result(&common::test_vec(INVALID_STRING));
}

#[test]
fn test_wtf8_string_is_invalid_utf8() {
    test_string_is_invalid_utf8(WTF8_STRING);
}

#[test]
fn test_wtf8_bytes() {
    assert_eq!(Ok(()), common::test_bytes(WTF8_STRING));
}

#[test]
fn test_wtf8_vec() {
    assert_eq!(Ok(()), common::test_vec(WTF8_STRING));
}

if_raw_str! {
    #[should_panic = "cannot split using an empty pattern"]
    #[test]
    fn test_split_by_empty() {
        let _ = RAW_WTF8_STRING.split("");
    }

    #[should_panic = "cannot split using an empty pattern"]
    #[test]
    fn test_split_empty_by_empty() {
        let _ = RawOsStr::from_str("").split("");
    }
}
