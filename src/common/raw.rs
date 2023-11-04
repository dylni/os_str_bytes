use std::fmt;
use std::fmt::Formatter;

use crate::RawOsStr;

if_conversions! {
    pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
        string.ends_with(suffix)
    }
}

if_conversions! {
    pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
        string.starts_with(prefix)
    }
}

pub(crate) fn debug(string: &RawOsStr, f: &mut Formatter<'_>) -> fmt::Result {
    for byte in string.as_encoded_bytes() {
        write!(f, "\\x{:02X}", byte)?;
    }
    Ok(())
}

#[cfg(feature = "uniquote")]
pub(crate) mod uniquote {
    use uniquote::Formatter;
    use uniquote::Quote;
    use uniquote::Result;

    use crate::RawOsStr;

    pub(crate) fn escape(string: &RawOsStr, f: &mut Formatter<'_>) -> Result {
        string.as_encoded_bytes().escape(f)
    }
}
