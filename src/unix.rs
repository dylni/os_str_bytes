use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;

use super::EncodingError;
use super::OsStrBytes;
use super::OsStringBytes;

impl OsStrBytes for OsStr {
    #[inline]
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError> {
        Ok(Cow::Borrowed(OsStrExt::from_bytes(string)))
    }

    #[inline]
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(OsStrExt::as_bytes(self))
    }
}

impl OsStringBytes for OsString {
    #[inline]
    fn from_bytes<TString>(string: TString) -> Result<Self, EncodingError>
    where
        TString: AsRef<[u8]>,
    {
        <OsStr as OsStrBytes>::from_bytes(string.as_ref()).map(Cow::into_owned)
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError> {
        Ok(OsStringExt::from_vec(string))
    }

    #[inline]
    fn into_vec(self) -> Vec<u8> {
        OsStringExt::into_vec(self)
    }
}
