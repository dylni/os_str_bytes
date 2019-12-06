#![cfg(unix)]

use os_str_bytes::EncodingError;

mod common;
use common::test_bytes;
use common::test_vec;
use common::INVALID_STRING;

#[test]
fn test_invalid_bytes() -> Result<(), EncodingError> {
    test_bytes(INVALID_STRING)
}

#[test]
fn test_invalid_vec() -> Result<(), EncodingError> {
    test_vec(INVALID_STRING)
}
