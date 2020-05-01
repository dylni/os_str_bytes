#![allow(dead_code)]

use std::borrow::Borrow;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

pub(super) const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

fn assert_bytes_eq<TString>(
    expected: &Result<TString::Owned, EncodingError>,
    result: &Result<Cow<'_, TString>, EncodingError>,
) where
    TString: Debug + PartialEq<TString> + ?Sized + ToOwned,
{
    assert_eq!(
        expected.as_ref().map(Borrow::borrow),
        result.as_ref().map(Borrow::borrow),
    );
}

pub(super) fn from_bytes(string: &[u8]) -> Result<OsString, EncodingError> {
    let os_string = OsString::from_bytes(string);
    assert_bytes_eq(&os_string, &OsStr::from_bytes(string));

    let path = PathBuf::from_bytes(string);
    assert_bytes_eq(&path, &Path::from_bytes(string));
    assert_eq!(os_string, path.map(PathBuf::into_os_string));

    os_string
}

pub(super) fn from_vec(string: Vec<u8>) -> Result<OsString, EncodingError> {
    let os_string = OsString::from_vec(string.clone());

    let path = PathBuf::from_vec(string);
    assert_eq!(os_string, path.map(PathBuf::into_os_string));

    os_string
}

pub(super) fn test_bytes(string: &[u8]) -> Result<(), EncodingError> {
    let os_string = from_bytes(string)?;
    assert_eq!(string.len(), os_string.len());
    assert_eq!(string, os_string.to_bytes().as_ref());
    Ok(())
}

pub(super) fn test_vec(string: &[u8]) -> Result<(), EncodingError> {
    let os_string = from_vec(string.to_vec())?;
    assert_eq!(string.len(), os_string.len());
    assert_eq!(string, os_string.into_vec().as_slice());
    Ok(())
}

pub(super) fn test_utf8_bytes(string: &str) {
    let os_string = OsString::from(string);
    let string = string.as_bytes();
    assert_eq!(Ok(&os_string), from_bytes(string).as_ref());
    assert_eq!(string, os_string.to_bytes().as_ref());
}

pub(super) fn test_utf8_vec(string: &str) {
    let os_string = OsString::from(string.to_string());
    let string = string.as_bytes();
    assert_eq!(Ok(&os_string), from_vec(string.to_vec()).as_ref());
    assert_eq!(string, os_string.into_vec().as_slice());
}
