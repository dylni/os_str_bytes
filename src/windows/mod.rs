if_conversions! {
    #[cfg(target_os = "uefi")]
    use std::os::uefi as sys;
    #[cfg(windows)]
    use std::os::windows as sys;
}

pub(super) mod convert_io;

if_conversions! {
    pub(super) mod convert;

    if_raw_str! {
        pub(super) mod raw;
    }
}
