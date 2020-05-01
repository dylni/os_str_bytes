mod common;
use common::test_bytes;

#[test]
fn test_edge_cases() {
    assert_eq!(Ok(()), test_bytes(b"\xED\xAB\xBE\xF4\x8D\xBC\x9A"));
}
