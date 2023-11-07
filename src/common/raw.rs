if_conversions! {
    pub(crate) fn ends_with(string: &[u8], suffix: &[u8]) -> bool {
        string.ends_with(suffix)
    }
}

if_conversions! {
    pub(crate) fn starts_with(string: &[u8], prefix: &[u8]) -> bool {
        string.starts_with(prefix)
    }
}
