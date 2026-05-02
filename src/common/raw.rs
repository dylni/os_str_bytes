use std::ffi::OsStr;
use std::slice;

use crate::OsUnit;

pub(crate) type OsUnits<'a> = slice::Iter<'a, u8>;

pub(crate) fn os_units(string: &OsStr) -> OsUnits<'_> {
    super::to_bytes(string).iter()
}

pub(crate) fn os_unit(unit: &u8) -> OsUnit {
    OsUnit((*unit).into())
}

if_conversions! {
    pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
        string.ends_with(suffix)
    }
}

if_conversions! {
    pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
        string.starts_with(prefix)
    }
}
