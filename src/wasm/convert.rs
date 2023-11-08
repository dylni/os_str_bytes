use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::result;
use std::str;
use std::str::Utf8Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EncodingError(Utf8Error);

impl Display for EncodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for EncodingError {}

pub(crate) type Result<T> = result::Result<T, EncodingError>;

macro_rules! expect_utf8 {
    ( $result:expr ) => {
        $result.expect(
            "platform string contains invalid UTF-8, which should not be \
             possible",
        )
    };
}

pub(crate) fn os_str_from_bytes(string: &[u8]) -> Result<Cow<'_, OsStr>> {
    str::from_utf8(string)
        .map(|x| Cow::Borrowed(OsStr::new(x)))
        .map_err(EncodingError)
}

pub(crate) fn os_str_to_bytes(string: &OsStr) -> Cow<'_, [u8]> {
    Cow::Borrowed(expect_utf8!(string.to_str()).as_bytes())
}

pub(crate) fn os_string_from_vec(string: Vec<u8>) -> Result<OsString> {
    String::from_utf8(string)
        .map(Into::into)
        .map_err(|x| EncodingError(x.utf8_error()))
}

pub(crate) fn os_string_into_vec(string: OsString) -> Vec<u8> {
    expect_utf8!(string.into_string()).into_bytes()
}
