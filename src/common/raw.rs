use std::fmt;
use std::fmt::Formatter;

use crate::RawOsStr;

if_not_nightly! {
    use super::Result;
}

if_not_nightly! {
    #[inline(always)]
    pub(crate) const fn is_continuation(_: u8) -> bool {
        unreachable!();
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
pub(crate) fn is_boundary(bytes: &[u8]) -> bool {
    debug_assert!(!bytes.is_empty());

    true
}

if_not_nightly! {
    #[inline(always)]
    pub(crate) fn validate_bytes(_: &[u8]) -> Result<()> {
        Ok(())
    }
}

if_not_nightly! {
    #[inline(always)]
    pub(crate) fn decode_code_point(_: &[u8]) -> u32 {
        unreachable!();
    }
}

pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
    string.ends_with(suffix)
}

pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
    string.starts_with(prefix)
}

#[cfg_attr(feature = "nightly", allow(deprecated), allow(unreachable_code))]
fn as_inner(string: &RawOsStr) -> &[u8] {
    if_nightly_return! {{
        string.as_os_str_bytes()
    }}
    string.as_raw_bytes()
}

pub(crate) fn debug(string: &RawOsStr, f: &mut Formatter<'_>) -> fmt::Result {
    for byte in as_inner(string) {
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
        super::as_inner(string).escape(f)
    }
}
