// These methods are necessarily inefficient, because they must revert encoding
// conversions performed by the standard library. However, there is currently
// no better alternative.

use std::borrow::Cow;
use std::char;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

use crate::EncodingError;
use crate::OsStrBytes;
use crate::OsStringBytes;

#[allow(clippy::module_inception)]
mod imp;

fn wide_to_wtf8<TString>(encoded_string: TString, length: usize) -> Vec<u8>
where
    TString: IntoIterator<Item = u16>,
{
    // https://github.com/rust-lang/rust/blob/49c68bd53f90e375bfb3cbba8c1c67a9e0adb9c0/src/libstd/sys_common/wtf8.rs#L183-L199

    let mut string = Vec::with_capacity(length);
    let mut buffer = [0; mem::size_of::<char>()];
    for ch in char::decode_utf16(encoded_string) {
        let unchecked_char = ch.unwrap_or_else(|surrogate| {
            let surrogate = surrogate.unpaired_surrogate().into();
            debug_assert!(surrogate <= u32::from(char::MAX));
            // SAFETY: https://docs.rs/os_str_bytes/#safety
            unsafe { char::from_u32_unchecked(surrogate) }
        });
        string.extend_from_slice(
            unchecked_char.encode_utf8(&mut buffer).as_bytes(),
        );
    }
    debug_assert_eq!(string.len(), length);
    string
}

fn wtf8_to_wide(string: &[u8]) -> Vec<u16> {
    // https://github.com/rust-lang/rust/blob/49c68bd53f90e375bfb3cbba8c1c67a9e0adb9c0/src/libstd/sys_common/wtf8.rs#L797-L813

    let mut string = string.iter();
    let mut encoded_string = Vec::new();
    let mut buffer = [0; 2];
    while let Some(code_point) = imp::next_code_point(&mut string) {
        debug_assert!(code_point <= u32::from(char::MAX));
        // SAFETY: https://docs.rs/os_str_bytes/#safety
        let unchecked_char = unsafe { char::from_u32_unchecked(code_point) };
        encoded_string
            .extend_from_slice(unchecked_char.encode_utf16(&mut buffer));
    }
    encoded_string
}

impl OsStrBytes for OsStr {
    #[inline]
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError> {
        Ok(Cow::Owned(OsStringBytes::from_bytes(string)?))
    }

    #[inline]
    unsafe fn from_bytes_unchecked(string: &[u8]) -> Cow<'_, Self> {
        Cow::Owned(OsStringBytes::from_bytes_unchecked(string))
    }

    #[inline]
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(wide_to_wtf8(OsStrExt::encode_wide(self), self.len()))
    }
}

impl OsStringBytes for OsString {
    #[allow(clippy::map_clone)]
    #[inline]
    fn from_bytes<TString>(string: TString) -> Result<Self, EncodingError>
    where
        TString: AsRef<[u8]>,
    {
        let string = string.as_ref();
        let encoded_string = wtf8_to_wide(string);
        if wide_to_wtf8(encoded_string.iter().map(|&x| x), string.len())
            == string
        {
            Ok(OsStringExt::from_wide(&encoded_string))
        } else {
            Err(EncodingError(()))
        }
    }

    #[inline]
    unsafe fn from_bytes_unchecked<TString>(string: TString) -> Self
    where
        TString: AsRef<[u8]>,
    {
        OsStringExt::from_wide(&wtf8_to_wide(string.as_ref()))
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError> {
        OsStringBytes::from_bytes(string)
    }

    #[inline]
    unsafe fn from_vec_unchecked(string: Vec<u8>) -> Self {
        OsStringBytes::from_bytes_unchecked(string)
    }

    #[inline]
    fn into_vec(self) -> Vec<u8> {
        OsStrBytes::to_bytes(self.as_os_str()).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::ffi::OsString;

    use crate::EncodingError;
    use crate::OsStrBytes;
    use crate::OsStringBytes;

    const INVALID_STRING: &[u8] = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";

    #[test]
    fn test_invalid_bytes() {
        assert_eq!(Err(EncodingError(())), OsStr::from_bytes(INVALID_STRING));
        assert_eq!(
            Err(EncodingError(())),
            OsString::from_bytes(INVALID_STRING),
        );
    }

    #[test]
    fn test_invalid_vec() {
        assert_eq!(
            Err(EncodingError(())),
            OsString::from_vec(INVALID_STRING.to_vec()),
        );
    }
}
