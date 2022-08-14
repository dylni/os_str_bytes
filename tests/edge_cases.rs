#![cfg(feature = "checked_conversions")]

mod common;

#[test]
fn test_complex() {
    assert_eq!(Ok(()), common::test_bytes(b"\xED\xAB\xBE\xF4\x8D\xBC\x9A"));
}
