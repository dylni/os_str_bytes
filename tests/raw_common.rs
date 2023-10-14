#![allow(dead_code)]
#![cfg(feature = "raw_os_str")]

#[path = "common.rs"]
#[macro_use]
mod common;

if_conversions! {
    use std::borrow::Cow;

    use lazy_static::lazy_static;

    use os_str_bytes::RawOsStr;

    use common::WTF8_STRING;
}

if_conversions! {
    lazy_static! {
        pub(crate) static ref RAW_WTF8_STRING: Cow<'static, RawOsStr> =
            RawOsStr::assert_cow_from_raw_bytes(WTF8_STRING);
    }
}
