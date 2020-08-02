use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

#[cfg(any(target_os = "hermit", target_os = "redox", unix))]
use std::os::unix as os;
#[cfg(any(target_env = "wasi", target_os = "wasi"))]
use std::os::wasi as os;

use os::ffi::OsStrExt;
use os::ffi::OsStringExt;

use super::EncodingError;
use super::OsStrBytes;
use super::OsStringBytes;

if_raw! {
    pub(super) mod raw;
}

impl OsStrBytes for OsStr {
    #[inline]
    fn from_bytes<TString>(
        string: &TString,
    ) -> Result<Cow<'_, Self>, EncodingError>
    where
        TString: AsRef<[u8]> + ?Sized,
    {
        Ok(Cow::Borrowed(OsStrExt::from_bytes(string.as_ref())))
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
        <OsStr as OsStrBytes>::from_bytes(&string).map(Cow::into_owned)
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
