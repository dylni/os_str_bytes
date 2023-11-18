use std::fmt::Debug;
use std::str;

use super::private;
use super::util::MAX_UTF8_LENGTH;

pub trait Encoded {
    fn __as_bytes(&self) -> &[u8] {
        self.__as_str().as_bytes()
    }

    fn __as_str(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct EncodedChar {
    buffer: [u8; MAX_UTF8_LENGTH],
    length: usize,
}

impl Encoded for EncodedChar {
    fn __as_str(&self) -> &str {
        // SAFETY: This slice was encoded from a character.
        unsafe { str::from_utf8_unchecked(&self.buffer[..self.length]) }
    }
}

impl Encoded for &str {
    fn __as_str(&self) -> &str {
        self
    }
}

/// Allows a type to be used for searching by [`RawOsStr`] and [`RawOsString`].
///
/// This trait is very similar to [`str::pattern::Pattern`], but its methods
/// are private and it is implemented for different types.
///
/// [`RawOsStr`]: super::RawOsStr
/// [`RawOsString`]: super::RawOsString
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub trait Pattern: private::Sealed {
    #[doc(hidden)]
    type __Encoded: Clone + Debug + Encoded;

    #[doc(hidden)]
    fn __encode(self) -> Self::__Encoded;
}

impl Pattern for char {
    type __Encoded = EncodedChar;

    fn __encode(self) -> Self::__Encoded {
        let mut encoded = EncodedChar {
            buffer: [0; MAX_UTF8_LENGTH],
            length: 0,
        };
        encoded.length = self.encode_utf8(&mut encoded.buffer).len();
        encoded
    }
}

impl Pattern for &str {
    type __Encoded = Self;

    fn __encode(self) -> Self::__Encoded {
        self
    }
}

impl<'a> Pattern for &'a String {
    type __Encoded = <&'a str as Pattern>::__Encoded;

    fn __encode(self) -> Self::__Encoded {
        (**self).__encode()
    }
}
