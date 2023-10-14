#![allow(dead_code)]

macro_rules! if_checked_conversions {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "checked_conversions")]
            $item
        )+
    };
}

if_checked_conversions! {
    use std::borrow::Cow;
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;
    use std::result;

    use os_str_bytes::EncodingError;
    use os_str_bytes::OsStrBytes;
    use os_str_bytes::OsStringBytes;
}

#[allow(unused_macros)]
macro_rules! if_conversions {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "conversions")]
            $item
        )+
    };
}

if_checked_conversions! {
    pub(crate) type Result<T> = result::Result<T, EncodingError>;
}

pub(crate) const WTF8_STRING: &[u8] = b"foo\xED\xA0\xBD\xF0\x9F\x92\xA9bar";

if_checked_conversions! {
    #[track_caller]
    fn test_from_bytes<'a, T, U, S>(result: &Result<U>, string: S)
    where
        S: Into<Cow<'a, [u8]>>,
        T: 'a + AsRef<OsStr> + OsStrBytes + ?Sized,
        U: AsRef<OsStr>,
    {
        assert_eq!(
            result.as_ref().map(AsRef::as_ref),
            T::from_raw_bytes(string).as_deref().map(AsRef::as_ref),
        );
    }

    pub(crate) fn from_bytes(string: &[u8]) -> Result<Cow<'_, OsStr>> {
        let os_string = OsStr::from_raw_bytes(string);

        test_from_bytes::<Path, _, _>(&os_string, string);

        os_string
    }

    pub(crate) fn from_vec(string: Vec<u8>) -> Result<OsString> {
        let os_string = OsString::from_raw_vec(string.clone());
        test_from_bytes::<OsStr, _, _>(&os_string, string.clone());

        let path = PathBuf::from_raw_vec(string.clone());
        test_from_bytes::<Path, _, _>(&path, string);
        assert_eq!(os_string, path.map(PathBuf::into_os_string));

        os_string
    }

    pub(crate) fn test_bytes(string: &[u8]) -> Result<()> {
        let os_string = from_bytes(string)?;
        assert_eq!(string.len(), os_string.len());
        assert_eq!(string, &*os_string.to_raw_bytes());
        Ok(())
    }

    pub(crate) fn test_vec(string: &[u8]) -> Result<()> {
        let os_string = from_vec(string.to_owned())?;
        assert_eq!(string.len(), os_string.len());
        assert_eq!(string, os_string.into_raw_vec());
        Ok(())
    }

    pub(crate) fn test_utf8_bytes(string: &str) {
        let os_string = OsStr::new(string);
        let string = string.as_bytes();
        assert_eq!(Ok(Cow::Borrowed(os_string)), from_bytes(string));
        assert_eq!(string, &*os_string.to_raw_bytes());
    }

    pub(crate) fn test_utf8_vec(string: &str) {
        let os_string = string.to_owned().into();
        let string = string.as_bytes();
        assert_eq!(Ok(&os_string), from_vec(string.to_owned()).as_ref());
        assert_eq!(string, os_string.into_raw_vec());
    }
}
