#![allow(dead_code)]
#![cfg(feature = "raw_os_str")]

use lazy_static::lazy_static;

use os_str_bytes::RawOsStr;

#[path = "common.rs"]
mod common;
use common::WTF8_STRING;

lazy_static! {
    pub(crate) static ref RAW_WTF8_STRING: &'static RawOsStr = {
        #[cfg_attr(feature = "nightly", allow(deprecated))]
        RawOsStr::assert_from_raw_bytes(WTF8_STRING)
    };
}
