#![allow(dead_code)]

use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

pub(crate) const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

pub(crate) fn from_bytes(string: &[u8]) -> Result<OsString, EncodingError> {
    let os_string = OsString::from_bytes(string)?;
    assert_eq!(os_string, OsStr::from_bytes(string)?);

    let path = PathBuf::from_bytes(string)?;
    assert_eq!(path, Path::from_bytes(string)?);
    assert_eq!(os_string, path);

    Ok(os_string)
}

pub(crate) fn from_vec(string: Vec<u8>) -> Result<OsString, EncodingError> {
    let os_string = OsString::from_vec(string.clone())?;

    let path = PathBuf::from_vec(string)?;
    assert_eq!(os_string, path);

    Ok(os_string)
}

pub(crate) fn test_bytes(string: &[u8]) -> Result<(), EncodingError> {
    let os_string = from_bytes(string)?;
    assert_eq!(string.len(), os_string.len());
    assert_eq!(string, os_string.to_bytes().as_ref());
    Ok(())
}

pub(crate) fn test_vec(string: &[u8]) -> Result<(), EncodingError> {
    let os_string = from_vec(string.to_vec())?;
    assert_eq!(string.len(), os_string.len());
    assert_eq!(string, os_string.into_vec().as_slice());
    Ok(())
}
