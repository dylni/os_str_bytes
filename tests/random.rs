use std::ffi::OsString;

use getrandom::getrandom;
use getrandom::Error as GetRandomError;

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
        use std::os::windows::ffi::OsStringExt;
        use std::slice;

        getrandom(as_mut_bytes(&mut buffer))?;
        return Ok(OsStringExt::from_wide(&buffer));

        fn as_mut_bytes(buffer: &mut [u16]) -> &mut [u8] {
            // SAFETY: [u16] can always be transmuted to two [u8] bytes.
            unsafe {
                slice::from_raw_parts_mut(
                    buffer.as_mut_ptr() as *mut u8,
                    buffer.len() * 2,
                )
            }
        }
    }
}

#[test]
fn test_random_bytes() -> Result<(), GetRandomError> {
    let os_string = random_os_string(RANDOM_BYTES_LENGTH)?;
    let string = os_string.to_bytes();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(&os_string), from_bytes(&string).as_ref());
    Ok(())
}

#[test]
fn test_random_vec() -> Result<(), GetRandomError> {
    let os_string = random_os_string(RANDOM_BYTES_LENGTH)?;
    let string = os_string.clone().into_vec();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(os_string), from_vec(string));
    Ok(())
}
