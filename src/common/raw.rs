#[inline(always)]
pub(crate) const fn is_continuation(_: u8) -> bool {
    false
}

#[inline(always)]
pub(crate) fn decode_code_point(_: &[u8]) -> u32 {
    unreachable!()
}

pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
    string.ends_with(suffix)
}

pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
    string.starts_with(prefix)
}
