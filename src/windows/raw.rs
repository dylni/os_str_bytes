use std::fmt;
use std::fmt::Formatter;

pub(crate) use crate::util::is_continuation;

use super::wtf8;
use super::wtf8::CodePoints;

pub(crate) fn decode_code_point(string: &[u8]) -> u32 {
    let mut code_points = CodePoints::new(string.iter().copied());
    let code_point = code_points
        .next()
        .expect("cannot parse code point from empty string")
        .expect("invalid string");
    assert_eq!(None, code_points.next(), "multiple code points found");
    code_point
}

pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
    wtf8::ends_with(string, suffix).unwrap_or(false)
}

pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
    wtf8::starts_with(string, prefix).unwrap_or(false)
}

pub(crate) fn debug(string: &[u8], f: &mut Formatter<'_>) -> fmt::Result {
    for wchar in wtf8::encode_wide(string) {
        write!(f, "\\u{{{:X}}}", wchar.expect("invalid string"))?;
    }
    Ok(())
}
