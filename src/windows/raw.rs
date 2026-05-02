use std::ffi::OsStr;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::iter::FusedIterator;

pub(crate) use crate::OsUnit as os_unit;

use super::os::ffi::EncodeWide;
use super::os::ffi::OsStrExt;

if_conversions! {
    pub(crate) use super::convert::ends_with;
    pub(crate) use super::convert::starts_with;
}

#[derive(Clone)]
pub(crate) struct OsUnits<'a>(EncodeWide<'a>);

impl Debug for OsUnits<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OsUnits").finish_non_exhaustive()
    }
}

impl FusedIterator for OsUnits<'_> {}

impl Iterator for OsUnits<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(crate) fn os_units(string: &OsStr) -> OsUnits<'_> {
    OsUnits(string.encode_wide())
}
