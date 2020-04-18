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
    fn from_bytes(string: &[u8]) -> Result<Cow<'_, Self>, EncodingError> {
        Ok(Cow::Owned(OsStringBytes::from_bytes(string)?))
    }

    #[inline]
    unsafe fn from_bytes_unchecked(string: &[u8]) -> Cow<'_, Self> {
        Cow::Owned(OsStringBytes::from_bytes_unchecked(string))
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
    unsafe fn from_bytes_unchecked<TString>(string: TString) -> Self
    where
        TString: AsRef<[u8]>,
    {
        OsStringExt::from_wide(&encode_wide(string).unwrap())
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
    use std::borrow::Borrow;
    use std::ffi::OsStr;
    use std::ffi::OsString;

    use getrandom::getrandom;
    use getrandom::Error as GetRandomError;

    use super::EncodingError;
    use super::OsStrBytes;
    use super::OsStringBytes;

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

    #[test]
    fn test_invalid() {
        test(b"\x0C\x83\xD7\x3E");
        test(b"\x19\xF7\x52\x84");
        test(b"\x70\xB8\x1F\x66");
        test(b"\x70\xFD\x80\x8E\x88");
        test(b"\x80");
        test(b"\x80\x80");
        test(b"\x80\x80\x80");
        test(b"\x81");
        test(b"\x88\xB4\xC7\x46");
        test(b"\x97\xCE\x06");
        test(b"\xC2\x00");
        test(b"\xC2\x7F");
        test(b"\xCD\x09\x95");
        test(b"\xCD\x43\x5F\xA0");
        test(b"\xD7\x69\xB2");
        test(b"\xE0\x94\xA8");
        test(b"\xE0\x9D\xA6\x12\xAE");
        test(b"\xE2\xAB\xFD\x51");
        test(b"\xE3\xC4");
        test(b"\xED\xA0\x80\xED\xB0\x80");
        test(b"\xF1");
        test(b"\xF1\x80");
        test(b"\xF1\x80\x80");
        test(b"\xF1\x80\x80\xF1");
        test(b"\xF5\x9E\xB1\x86");
        test(b"\xFB");
        test(b"\xFB\x80");
        test(b"\xFB\x80\x80");
        test(b"\xFB\x80\x80\x80");
        test(b"\xFF");
        test(b"\xFF\x80");
        test(b"\xFF\x80\x80");
        test(b"\xFF\x80\x80\x80");
        test(b"\xFF\x86\x85\x83");

        fn test(string: &[u8]) {
            assert_eq!(Err(EncodingError(())), OsStr::from_bytes(string));
        }
    }

    #[test]
    fn test_random() -> Result<(), GetRandomError> {
        for _ in 1..1024 {
            let mut string = vec![0; 16];
            getrandom(&mut string)?;
            if let Ok(os_string) = OsStr::from_bytes(&string) {
                let encoded_string = os_string.to_bytes();
                assert_eq!(string, Borrow::<[u8]>::borrow(&encoded_string));
            }
        }
        Ok(())
    }
}
