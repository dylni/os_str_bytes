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

mod wtf8;
use wtf8::DecodeWide;
use wtf8::EncodeWide;

fn encode_wide<TString>(string: TString) -> Result<Vec<u16>, EncodingError>
where
    TString: AsRef<[u8]>,
{
    let string = string.as_ref();
    #[allow(clippy::map_clone)]
    let encoder = EncodeWide::new(string.iter().map(|&x| x));

    // Collecting an iterator into a result ignores the size hint:
    // https://github.com/rust-lang/rust/issues/48994
    let mut encoded_string = Vec::with_capacity(encoder.size_hint().0);
    for wchar in encoder {
        encoded_string.push(wchar.map_err(EncodingError)?);
    }
    Ok(encoded_string)
}

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
        encode_wide(string).map(|x| OsStringExt::from_wide(&x))
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
        test_byte_error(b"\x0C\x83\xD7\x3E", b'\x83');
        test_byte_error(b"\x19\xF7\x52\x84", b'\x52');
        test_byte_error(b"\x70\xB8\x1F\x66", b'\xB8');
        test_code_point_error(b"\x70\xFD\x80\x8E\x88", 0x34_0388);
        test_byte_error(b"\x80", b'\x80');
        test_byte_error(b"\x80\x80", b'\x80');
        test_byte_error(b"\x80\x80\x80", b'\x80');
        test_byte_error(b"\x81", b'\x81');
        test_byte_error(b"\x88\xB4\xC7\x46", b'\x88');
        test_byte_error(b"\x97\xCE\x06", b'\x97');
        test_byte_error(b"\xC2\x00", b'\x00');
        test_byte_error(b"\xC2\x7F", b'\x7F');
        test_byte_error(b"\xCD\x09\x95", b'\x09');
        test_byte_error(b"\xCD\x43\x5F\xA0", b'\x43');
        test_byte_error(b"\xD7\x69\xB2", b'\x69');
        test_code_point_error(b"\xE0\x94\xA8", 0x528);
        test_code_point_error(b"\xE0\x9D\xA6\x12\xAE", 0x766);
        test_byte_error(b"\xE2\xAB\xFD\x51", b'\xFD');
        test_byte_error(b"\xE3\xC4", b'\xC4');
        test_code_point_error(b"\xED\xA0\x80\xED\xB0\x80", 0xDC00);
        test_end_error(b"\xF1");
        test_end_error(b"\xF1\x80");
        test_end_error(b"\xF1\x80\x80");
        test_byte_error(b"\xF1\x80\x80\xF1", b'\xF1');
        test_code_point_error(b"\xF5\x9E\xB1\x86", 0x15_EC46);
        test_end_error(b"\xFB");
        test_end_error(b"\xFB\x80");
        test_end_error(b"\xFB\x80\x80");
        test_code_point_error(b"\xFB\x80\x80\x80", 0x2C_0000);
        test_end_error(b"\xFF");
        test_end_error(b"\xFF\x80");
        test_end_error(b"\xFF\x80\x80");
        test_code_point_error(b"\xFF\x80\x80\x80", 0x3C_0000);
        test_code_point_error(b"\xFF\x86\x85\x83", 0x3C_6143);

        fn test(string: &[u8], error: EncodingError) {
            use crate::EncodingError;

            assert_eq!(
                Err(error),
                OsStr::from_bytes(string).map_err(|EncodingError(x)| x),
            );
        }

        fn test_byte_error(string: &[u8], byte: u8) {
            test(string, EncodingError::Byte(byte));
        }

        fn test_code_point_error(string: &[u8], code_point: u32) {
            test(string, EncodingError::CodePoint(code_point));
        }

        fn test_end_error(string: &[u8]) {
            test(string, EncodingError::End());
        }
    }
}
