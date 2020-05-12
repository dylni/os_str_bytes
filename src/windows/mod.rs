// These methods are necessarily inefficient, because they must revert encoding
// conversions performed by the standard library. However, there is currently
// no better alternative.

use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;

use super::EncodingError;
use super::OsStrBytes;
use super::OsStringBytes;

if_raw! {
    pub(crate) mod raw;
}

mod wtf8;
use wtf8::DecodeWide;
use wtf8::EncodeWide;

impl OsStrBytes for OsStr {
    #[inline]
    fn from_bytes<TString>(
        string: &TString,
    ) -> Result<Cow<'_, Self>, EncodingError>
    where
        TString: AsRef<[u8]> + ?Sized,
    {
        Ok(Cow::Owned(OsStringBytes::from_bytes(string)?))
    }

    #[inline]
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(DecodeWide::new(OsStrExt::encode_wide(self)).collect())
    }
}

impl OsStringBytes for OsString {
    #[inline]
    fn from_bytes<TString>(string: TString) -> Result<Self, EncodingError>
    where
        TString: AsRef<[u8]>,
    {
        let encoder = EncodeWide::new(string.as_ref().iter().map(|&x| x));

        // Collecting an iterator into a result ignores the size hint:
        // https://github.com/rust-lang/rust/issues/48994
        let mut encoded_string = Vec::with_capacity(encoder.size_hint().0);
        for wchar in encoder {
            encoded_string.push(wchar.map_err(EncodingError)?);
        }
        Ok(OsStringExt::from_wide(&encoded_string))
    }

    #[inline]
    fn from_vec(string: Vec<u8>) -> Result<Self, EncodingError> {
        OsStringBytes::from_bytes(string)
    }

    #[inline]
    fn into_vec(self) -> Vec<u8> {
        OsStrBytes::to_bytes(self.as_os_str()).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use crate::error::EncodingError;

    use super::OsStrBytes;

    #[test]
    fn test_invalid() {
        use EncodingError::Byte;
        use EncodingError::CodePoint;
        use EncodingError::End;

        test_error(Byte(b'\x83'), b"\x0C\x83\xD7\x3E");
        test_error(Byte(b'\x52'), b"\x19\xF7\x52\x84");
        test_error(Byte(b'\xB8'), b"\x70\xB8\x1F\x66");
        test_error(CodePoint(0x34_0388), b"\x70\xFD\x80\x8E\x88");
        test_error(Byte(b'\x80'), b"\x80");
        test_error(Byte(b'\x80'), b"\x80\x80");
        test_error(Byte(b'\x80'), b"\x80\x80\x80");
        test_error(Byte(b'\x81'), b"\x81");
        test_error(Byte(b'\x88'), b"\x88\xB4\xC7\x46");
        test_error(Byte(b'\x97'), b"\x97\xCE\x06");
        test_error(Byte(b'\x00'), b"\xC2\x00");
        test_error(Byte(b'\x7F'), b"\xC2\x7F");
        test_error(Byte(b'\x09'), b"\xCD\x09\x95");
        test_error(Byte(b'\x43'), b"\xCD\x43\x5F\xA0");
        test_error(Byte(b'\x69'), b"\xD7\x69\xB2");
        test_error(CodePoint(0x528), b"\xE0\x94\xA8");
        test_error(CodePoint(0x766), b"\xE0\x9D\xA6\x12\xAE");
        test_error(Byte(b'\xFD'), b"\xE2\xAB\xFD\x51");
        test_error(Byte(b'\xC4'), b"\xE3\xC4");
        test_error(CodePoint(0xDC00), b"\xED\xA0\x80\xED\xB0\x80");
        test_error(End(), b"\xF1");
        test_error(End(), b"\xF1\x80");
        test_error(End(), b"\xF1\x80\x80");
        test_error(Byte(b'\xF1'), b"\xF1\x80\x80\xF1");
        test_error(CodePoint(0x15_EC46), b"\xF5\x9E\xB1\x86");
        test_error(End(), b"\xFB");
        test_error(End(), b"\xFB\x80");
        test_error(End(), b"\xFB\x80\x80");
        test_error(CodePoint(0x2C_0000), b"\xFB\x80\x80\x80");
        test_error(End(), b"\xFF");
        test_error(End(), b"\xFF\x80");
        test_error(End(), b"\xFF\x80\x80");
        test_error(CodePoint(0x3C_0000), b"\xFF\x80\x80\x80");
        test_error(CodePoint(0x3C_6143), b"\xFF\x86\x85\x83");

        fn test_error(error: EncodingError, string: &[u8]) {
            use crate::EncodingError;

            assert_eq!(
                Err(error),
                OsStr::from_bytes(string).map_err(|EncodingError(x)| x),
            );
        }
    }
}
