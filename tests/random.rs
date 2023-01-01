#![cfg(feature = "checked_conversions")]

use std::borrow::Cow;
use std::ffi::OsStr;

use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

mod common;

mod random_common;
use random_common::ITERATIONS;
use random_common::SMALL_LENGTH;

const LARGE_LENGTH: usize = 1024;

#[test]
fn test_bytes() {
    let os_string = random_common::fastrand_os_string(LARGE_LENGTH);
    let string = os_string.to_raw_bytes();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(Cow::Borrowed(&*os_string)), common::from_bytes(&string));
}

#[test]
fn test_vec() {
    let os_string = random_common::fastrand_os_string(LARGE_LENGTH);
    let string = os_string.clone().into_raw_vec();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(os_string), common::from_vec(string));
}

#[test]
fn test_lossless() {
    for _ in 0..ITERATIONS {
        let mut string = vec![0; SMALL_LENGTH];
        random_common::fastrand_fill(&mut string);
        if let Ok(os_string) = OsStr::from_raw_bytes(&string) {
            assert_eq!(string, &*os_string.to_raw_bytes());
        }
    }
}
