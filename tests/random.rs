use std::ffi::OsString;

use getrandom::getrandom;
use getrandom::Error as GetRandomError;

use os_str_bytes::EncodingError;
use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

mod common;
use common::from_bytes;
use common::from_vec;

const RANDOM_BYTES_LENGTH: usize = 1024;

fn random_os_string(buffer_length: usize) -> Result<OsString, GetRandomError> {
    let mut buffer = vec![0; buffer_length];
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;

        getrandom(&mut buffer)?;
        Ok(OsStringExt::from_vec(buffer))
    }
    #[cfg(windows)]
    {
        use std::mem;
        use std::os::windows::ffi::OsStringExt;

        // SAFETY: These bytes are random, so their values are arbitrary.
        getrandom(unsafe {
            #[allow(clippy::transmute_ptr_to_ptr)]
            mem::transmute::<&mut [u16], &mut [u8]>(&mut buffer)
        })?;
        Ok(OsStringExt::from_wide(&buffer))
    }
}

#[test]
fn test_random_bytes() -> Result<(), EncodingError> {
    let os_string = random_os_string(RANDOM_BYTES_LENGTH).unwrap();
    let string = os_string.to_bytes();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(os_string, from_bytes(&string)?);
    Ok(())
}

#[test]
fn test_random_vec() -> Result<(), EncodingError> {
    let os_string = random_os_string(RANDOM_BYTES_LENGTH).unwrap();
    let string = os_string.clone().into_vec();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(os_string, from_vec(string)?);
    Ok(())
}
