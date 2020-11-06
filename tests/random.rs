use std::ffi::OsStr;
use std::ffi::OsString;

use getrandom::getrandom;

use os_str_bytes::OsStrBytes;
use os_str_bytes::OsStringBytes;

mod common;
use common::from_bytes;
use common::from_vec;

const SMALL_LENGTH: usize = 16;

const LARGE_LENGTH: usize = 1024;

const ITERATIONS: usize = 1024;

fn random_os_string(
    buffer_length: usize,
) -> Result<OsString, getrandom::Error> {
    let mut buffer = vec![0; buffer_length];
    #[cfg(not(windows))]
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
fn test_random_bytes() -> Result<(), getrandom::Error> {
    let os_string = random_os_string(LARGE_LENGTH)?;
    let string = os_string.to_bytes();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(&os_string), from_bytes(&string).as_ref());
    Ok(())
}

#[test]
fn test_random_vec() -> Result<(), getrandom::Error> {
    let os_string = random_os_string(LARGE_LENGTH)?;
    let string = os_string.clone().into_vec();
    assert_eq!(os_string.len(), string.len());
    assert_eq!(Ok(os_string), from_vec(string));
    Ok(())
}

#[test]
fn test_lossless() -> Result<(), getrandom::Error> {
    for _ in 0..ITERATIONS {
        let mut string = vec![0; SMALL_LENGTH];
        getrandom(&mut string)?;
        if let Ok(os_string) = OsStr::from_bytes(&string) {
            let encoded_string = os_string.to_bytes();
            assert_eq!(string, &*encoded_string);
        }
    }
    Ok(())
}

#[cfg(feature = "raw")]
#[test]
fn test_raw() -> Result<(), getrandom::Error> {
    use os_str_bytes::raw;

    macro_rules! test {
        ( $result:expr , $raw_fn:ident ( $string:expr , $substring:expr ) ) => {
            assert_eq!(
                $result,
                raw::$raw_fn(&$string, &$substring),
                concat!("raw::", stringify!($raw_fn), "({:?}, {:?})"),
                $string,
                $substring,
            );
        };
    }

    for _ in 0..ITERATIONS {
        let mut string = random_os_string(SMALL_LENGTH)?;
        let prefix = string.to_bytes().into_owned();
        let suffix = random_os_string(SMALL_LENGTH)?;
        string.push(&suffix);

        let string = string.into_vec();
        let suffix = suffix.into_vec();

        test!(true, ends_with(string, suffix));
        test!(true, starts_with(string, prefix));

        if prefix != suffix {
            test!(false, ends_with(string, prefix));
            test!(false, starts_with(string, suffix));
        }
    }
    Ok(())
}
