// These methods are necessarily inefficient, because they must revert encoding
// conversions performed by the standard library. However, there is currently
// no better alternative.

use std::borrow::Cow;
use std::char;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

use crate::EncodingError;
use crate::OsStrBytes;
use crate::OsStringBytes;

#[allow(clippy::module_inception)]
mod imp;

// UTF-8 ranges and tags for encoding characters
// From Rust's libcore/char/mod.rs
const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;
const MAX_ONE_B: u32 = 0x80;
const MAX_TWO_B: u32 = 0x800;
const MAX_THREE_B: u32 = 0x10000;

// From Rust's libcore/char/methods.rs (char::len_utf8)
fn len_wtf8(code: u32) -> usize {
    if code < MAX_ONE_B {
        1
    } else if code < MAX_TWO_B {
        2
    } else if code < MAX_THREE_B {
        3
    } else {
        4
    }
}

// From Rust's libcore/char/methods.rs (char::encode_utf8)
fn encode_wtf8(code: u32, dst: &mut [u8]) -> &mut [u8] {
    let len = len_wtf8(code);
    match (len, &mut dst[..]) {
        (1, [a, ..]) => {
            *a = code as u8;
        }
        (2, [a, b, ..]) => {
            *a = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
            *b = (code & 0x3F) as u8 | TAG_CONT;
        }
        (3, [a, b, c, ..]) => {
            *a = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
            *b = (code >> 6 & 0x3F) as u8 | TAG_CONT;
            *c = (code & 0x3F) as u8 | TAG_CONT;
        }
        (4, [a, b, c, d, ..]) => {
            *a = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
            *b = (code >> 12 & 0x3F) as u8 | TAG_CONT;
            *c = (code >> 6 & 0x3F) as u8 | TAG_CONT;
            *d = (code & 0x3F) as u8 | TAG_CONT;
        }
        _ => unreachable!(),
    };
    &mut dst[..len]
}

// From Rust's libcore/char/methods.rs (char::encode_utf16)
fn encode_wide(mut code: u32, dst: &mut [u16]) -> &mut [u16] {
    if (code & 0xFFFF) == code && !dst.is_empty() {
        // The BMP falls through (assuming non-surrogate, as it should)
        dst[0] = code as u16;
        &mut dst[..1]
    } else if dst.len() >= 2 {
        // Supplementary planes break into surrogates.
        code -= 0x1_0000;
        dst[0] = 0xD800 | ((code >> 10) as u16);
        dst[1] = 0xDC00 | ((code as u16) & 0x3FF);
        dst
    } else {
        unreachable!()
    }
}

fn wide_to_wtf8<TString>(encoded_string: TString, length: usize) -> Vec<u8>
where
    TString: IntoIterator<Item = u16>,
{
    // https://github.com/rust-lang/rust/blob/49c68bd53f90e375bfb3cbba8c1c67a9e0adb9c0/src/libstd/sys_common/wtf8.rs#L183-L199

    let mut string = Vec::with_capacity(length);
    let mut buffer = [0; 4];
    for ch in char::decode_utf16(encoded_string) {
        let ch = ch
            .map(u32::from)
            .unwrap_or_else(|surrogate| surrogate.unpaired_surrogate().into());
        string.extend_from_slice(encode_wtf8(ch, &mut buffer));
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
        encoded_string.extend_from_slice(encode_wide(code_point, &mut buffer));
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
