use std::fmt;
use std::fmt::Formatter;

use crate::RawOsStr;

#[allow(dead_code)]
#[path = "../common/raw.rs"]
mod common_raw;
#[cfg(feature = "uniquote")]
pub(crate) use common_raw::uniquote;

if_conversions! {
    pub(crate) use common_raw::ends_with;
    pub(crate) use common_raw::starts_with;
}

pub(crate) fn debug(string: &RawOsStr, _: &mut Formatter<'_>) -> fmt::Result {
    assert!(string.is_empty());
    Ok(())
}
