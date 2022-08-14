#![allow(dead_code)]
#![cfg(feature = "raw_os_str")]

use std::mem;

use os_str_bytes::RawOsStr;

#[path = "common.rs"]
mod common;
use common::WTF8_STRING;

// SAFETY: This string is valid in WTF-8. This implementation detail can only
// be assumed by this crate.
#[cfg(any(unix, windows))]
pub(crate) const RAW_WTF8_STRING: &RawOsStr =
    unsafe { mem::transmute(WTF8_STRING) };
