if_nightly! {
    use std::str;
}

pub(super) const BYTE_SHIFT: u8 = 6;

pub(super) const CONT_MASK: u8 = (1 << BYTE_SHIFT) - 1;

pub(super) const CONT_TAG: u8 = 0b1000_0000;

pub(super) const fn is_continuation(byte: u8) -> bool {
    byte & !CONT_MASK == CONT_TAG
}

if_raw_str! {
    #[cfg_attr(feature = "nightly", allow(unreachable_code))]
    pub(super) fn is_boundary(bytes: &[u8]) -> bool {
        debug_assert!(!bytes.is_empty());

        if_nightly_return! {{
            str::from_utf8(&bytes[..bytes.len().min(4)])
                .err()
                .map(|x| x.valid_up_to() != 0)
                .unwrap_or(true)
        }}
        !is_continuation(bytes[0])
    }
}
