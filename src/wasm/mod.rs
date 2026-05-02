if_os_conversions! {
    use std::ffi::OsStr;
}

if_os_conversions! {
    macro_rules! expect_utf8 {
        ( $result:expr ) => {
            $result.expect(
                "platform string contains invalid UTF-8, which should not be \
                 possible",
            )
        };
    }
}

#[path = "../windows/convert_io.rs"]
pub(super) mod convert_io;

if_conversions! {
    pub(super) mod convert;
}

if_raw_str! {
    #[path = "../common/raw.rs"]
    pub(super) mod raw;
}

if_os_conversions! {
    fn to_bytes(string: &OsStr) -> &[u8] {
        expect_utf8!(string.to_str()).as_bytes()
    }
}
