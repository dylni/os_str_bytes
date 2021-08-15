use super::private;

pub trait Encoder {
    fn __encode(&mut self) -> &[u8];
}

pub struct ByteEncoder([u8; 1]);

impl Encoder for ByteEncoder {
    fn __encode(&mut self) -> &[u8] {
        &self.0
    }
}

pub struct CharEncoder {
    buffer: [u8; 4],
    ch: char,
}

impl Encoder for CharEncoder {
    fn __encode(&mut self) -> &[u8] {
        self.ch.encode_utf8(&mut self.buffer).as_bytes()
    }
}

impl Encoder for &str {
    fn __encode(&mut self) -> &[u8] {
        self.as_bytes()
    }
}

/// Allows a type to be used for searching by [`RawOsStr`] and [`RawOsString`].
///
/// This trait is very similar to [`str::pattern::Pattern`], but its methods
/// are private and it is implemented for different types.
///
/// [`RawOsStr`]: super::RawOsStr
/// [`RawOsString`]: super::RawOsString
/// [`str::pattern::Pattern`]: ::std::str::pattern::Pattern
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub trait Pattern: private::Sealed {
    #[doc(hidden)]
    type __Encoder: Encoder;

    #[doc(hidden)]
    fn __into_encoder(self) -> Self::__Encoder;
}

impl Pattern for char {
    #[doc(hidden)]
    type __Encoder = CharEncoder;

    #[doc(hidden)]
    fn __into_encoder(self) -> Self::__Encoder {
        CharEncoder {
            buffer: [0; 4],
            ch: self,
        }
    }
}

impl Pattern for &str {
    #[doc(hidden)]
    type __Encoder = Self;

    #[doc(hidden)]
    fn __into_encoder(self) -> Self::__Encoder {
        self
    }
}

impl<'a> Pattern for &'a String {
    #[doc(hidden)]
    type __Encoder = &'a str;

    #[doc(hidden)]
    fn __into_encoder(self) -> Self::__Encoder {
        self
    }
}

impl Pattern for u8 {
    #[doc(hidden)]
    type __Encoder = ByteEncoder;

    #[doc(hidden)]
    fn __into_encoder(self) -> Self::__Encoder {
        assert!(self.is_ascii(), "byte pattern is not ASCII");

        ByteEncoder([self])
    }
}
