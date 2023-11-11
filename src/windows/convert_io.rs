use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::str;

pub(crate) fn os_str_from_bytes(string: &[u8]) -> Option<&OsStr> {
    str::from_utf8(string).map(OsStr::new).ok()
}

pub(crate) fn os_str_to_bytes(string: &OsStr) -> Option<&'_ [u8]> {
    string.to_str().map(str::as_bytes)
}

pub(crate) fn os_str_to_bytes_lossy(string: &OsStr) -> Cow<'_, [u8]> {
    match string.to_string_lossy() {
        Cow::Borrowed(string) => Cow::Borrowed(string.as_bytes()),
        Cow::Owned(string) => Cow::Owned(string.into_bytes()),
    }
}

pub(crate) fn os_string_from_vec(string: Vec<u8>) -> Option<OsString> {
    String::from_utf8(string).ok().map(Into::into)
}

pub(crate) fn os_string_into_vec(string: OsString) -> Option<Vec<u8>> {
    string.into_string().ok().map(String::into_bytes)
}

pub(crate) fn os_string_into_vec_lossy(string: OsString) -> Vec<u8> {
    let string = string.into_encoded_bytes();
    match String::from_utf8_lossy(&string) {
        // SAFETY: This slice was validated to be UTF-8.
        Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(string) },
        Cow::Owned(string) => string,
    }
    .into_bytes()
}
