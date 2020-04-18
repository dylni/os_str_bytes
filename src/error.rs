use std::borrow::Cow;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg_attr(unix, allow(dead_code))]
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum EncodingError {
    Byte(u8),
    CodePoint(u32),
    End(),
}

impl EncodingError {
    fn position(&self) -> Cow<'_, str> {
        // Variants are not recognized on type aliases in older versions:
        // https://github.com/rust-lang/rust/pull/61682
        match self {
            EncodingError::Byte(byte) => {
                Cow::Owned(format!("byte b'\\x{:02X}'", byte))
            }
            EncodingError::CodePoint(code_point) => {
                Cow::Owned(format!("code point U+{:04X}", code_point))
            }
            EncodingError::End() => Cow::Borrowed("end of string"),
        }
    }
}

impl Display for EncodingError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "os_str_bytes: byte sequence is not representable in the platform \
            encoding; error at {}",
            self.position(),
        )
    }
}

impl Error for EncodingError {}
