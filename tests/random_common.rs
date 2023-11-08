#![allow(dead_code)]

use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;
#[cfg(windows)]
use std::slice;

use fastrand::Rng;

pub(crate) const SMALL_LENGTH: usize = 16;

pub(crate) const ITERATIONS: usize = 1024;

pub(crate) fn fastrand_os_string(buffer_length: usize) -> OsString {
    let mut rng = Rng::new();
    let mut buffer = vec![0; buffer_length];
    #[cfg(unix)]
    {
        rng.fill(&mut buffer);
        OsString::from_vec(buffer)
    }
    #[cfg(windows)]
    {
        fn as_mut_bytes(buffer: &mut [u16]) -> &mut [u8] {
            // SAFETY: [u16] can always be transmuted to two [u8] bytes.
            unsafe {
                slice::from_raw_parts_mut(
                    buffer.as_mut_ptr().cast(),
                    buffer.len() * 2,
                )
            }
        }

        rng.fill(as_mut_bytes(&mut buffer));
        OsString::from_wide(&buffer)
    }
}
