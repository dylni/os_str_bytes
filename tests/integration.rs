use std::ffi::OsStr;
use std::ffi::OsString;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

const UTF8_STRING: &str = "string";

#[test]
fn test_empty_bytes() -> Result<(), EncodingError> {
    assert_eq!(OsString::new(), OsStr::from_bytes(&[])?);
    assert_eq!(OsString::new(), OsString::from_bytes([])?);
    // Assist type inference.
    assert_eq!(&[b'\0'; 0], OsString::new().as_os_str().to_bytes().as_ref());
    Ok(())
}

#[test]
fn test_empty_vec() -> Result<(), EncodingError> {
    assert_eq!(0, OsString::from_vec(Vec::new())?.len());
    assert_eq!(Vec::<u8>::new(), OsString::new().into_vec());
    Ok(())
}

#[test]
fn test_utf8_bytes() -> Result<(), EncodingError> {
    let os_str = OsString::from(UTF8_STRING);
    let os_str = os_str.as_os_str();
    assert_eq!(os_str, OsStr::from_bytes(UTF8_STRING.as_bytes())?);
    assert_eq!(os_str, OsString::from_bytes(UTF8_STRING)?);
    assert_eq!(UTF8_STRING.as_bytes(), os_str.to_bytes().as_ref());
    Ok(())
}

#[test]
fn test_utf8_vec() -> Result<(), EncodingError> {
    let os_string = OsString::from(UTF8_STRING);
    assert_eq!(
        os_string,
        OsString::from_vec(UTF8_STRING.to_string().into_bytes())?,
    );
    assert_eq!(UTF8_STRING.to_string().into_bytes(), os_string.into_vec());
    Ok(())
}
