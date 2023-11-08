#![allow(dead_code)]
#![cfg(feature = "raw_os_str")]

#[path = "common.rs"]
#[macro_use]
mod common;

if_conversions! {
    use std::borrow::Cow;
    use std::ffi::OsStr;

    use lazy_static::lazy_static;

    use os_str_bytes::OsStrBytes;

    use common::WTF8_STRING;
}

if_conversions! {
    lazy_static! {
        pub(crate) static ref WTF8_OS_STRING: Cow<'static, OsStr> =
            OsStr::assert_from_raw_bytes(WTF8_STRING);
    }
}
