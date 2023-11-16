if_conversions! {
    #[cfg(target_os = "uefi")]
    use std::os::uefi as os;
    #[cfg(windows)]
    use std::os::windows as os;
}

pub(super) mod convert_io;

if_conversions! {
    pub(super) mod convert;

    if_raw_str! {
        pub(super) mod raw;
    }
}
