use std::fmt;
use std::fmt::Formatter;
use std::os::windows::ffi::OsStrExt;

use crate::RawOsStr;

if_conversions! {
    pub(crate) use super::convert::ends_with;
    pub(crate) use super::convert::starts_with;
}

pub(crate) fn encode_wide(
    string: &RawOsStr,
) -> impl '_ + Iterator<Item = u16> {
    string.as_os_str().encode_wide()
}

pub(crate) fn debug(string: &RawOsStr, f: &mut Formatter<'_>) -> fmt::Result {
    for wchar in encode_wide(string) {
        write!(f, "\\u{{{:X}}}", wchar)?;
    }
    Ok(())
}

#[cfg(feature = "uniquote")]
pub(crate) mod uniquote {
    use uniquote::Formatter;
    use uniquote::Result;

    use crate::RawOsStr;

    pub(crate) fn escape(string: &RawOsStr, f: &mut Formatter<'_>) -> Result {
        f.escape_utf16(super::encode_wide(string))
    }
}
