use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

use super::os::ffi::OsStrExt;
use super::os::ffi::OsStringExt;

pub(crate) fn os_str_from_bytes(string: &[u8]) -> Option<&OsStr> {
    Some(OsStr::from_bytes(string))
}

pub(crate) fn os_str_to_bytes(string: &OsStr) -> Option<&'_ [u8]> {
    Some(string.as_bytes())
}

pub(crate) fn os_str_to_bytes_lossy(string: &OsStr) -> Cow<'_, [u8]> {
    Cow::Borrowed(string.as_bytes())
}

pub(crate) fn os_string_from_vec(string: Vec<u8>) -> Option<OsString> {
    Some(OsString::from_vec(string))
}

pub(crate) fn os_string_into_vec(string: OsString) -> Option<Vec<u8>> {
    Some(string.into_vec())
}

pub(crate) fn os_string_into_vec_lossy(string: OsString) -> Vec<u8> {
    string.into_vec()
}
