#[cfg(all(target_vendor = "fortanix", target_env = "sgx"))]
use std::os::fortanix_sgx as os;
#[cfg(target_os = "hermit")]
use std::os::hermit as os;
#[cfg(target_os = "solid_asp3")]
use std::os::solid as os;
#[cfg(unix)]
use std::os::unix as os;
#[cfg(target_os = "wasi")]
use std::os::wasi as os;
#[cfg(target_os = "xous")]
use std::os::xous as os;

pub(super) mod convert_io;

if_conversions! {
    pub(super) mod convert;

    if_raw_str! {
        pub(super) mod raw;
    }
}
