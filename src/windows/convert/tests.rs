use super::EncodingError;

#[test]
fn test_invalid() {
    use EncodingError::Byte;
    use EncodingError::CodePoint;
    use EncodingError::End;

    #[track_caller]
    fn test(error: EncodingError, string: &[u8]) {
        assert_eq!(Err(error), super::from_bytes(string));
    }

    test(Byte(b'\x83'), b"\x0C\x83\xD7\x3E");
    test(Byte(b'\x52'), b"\x19\xF7\x52\x84");
    test(Byte(b'\xB8'), b"\x70\xB8\x1F\x66");
    test(CodePoint(0x34_0388), b"\x70\xFD\x80\x8E\x88");
    test(Byte(b'\x80'), b"\x80");
    test(Byte(b'\x80'), b"\x80\x80");
    test(Byte(b'\x80'), b"\x80\x80\x80");
    test(Byte(b'\x81'), b"\x81");
    test(Byte(b'\x88'), b"\x88\xB4\xC7\x46");
    test(Byte(b'\x97'), b"\x97\xCE\x06");
    test(Byte(b'\x00'), b"\xC2\x00");
    test(Byte(b'\x7F'), b"\xC2\x7F");
    test(Byte(b'\x09'), b"\xCD\x09\x95");
    test(Byte(b'\x43'), b"\xCD\x43\x5F\xA0");
    test(Byte(b'\x69'), b"\xD7\x69\xB2");
    test(CodePoint(0x528), b"\xE0\x94\xA8");
    test(CodePoint(0x766), b"\xE0\x9D\xA6\x12\xAE");
    test(Byte(b'\xFD'), b"\xE2\xAB\xFD\x51");
    test(Byte(b'\xC4'), b"\xE3\xC4");
    test(CodePoint(0xDC00), b"\xED\xA0\x80\xED\xB0\x80");
    test(End(), b"\xF1");
    test(End(), b"\xF1\x80");
    test(End(), b"\xF1\x80\x80");
    test(Byte(b'\xF1'), b"\xF1\x80\x80\xF1");
    test(CodePoint(0x11_09CC), b"\xF4\x90\xA7\x8C");
    test(CodePoint(0x15_EC46), b"\xF5\x9E\xB1\x86");
    test(End(), b"\xFB");
    test(End(), b"\xFB\x80");
    test(End(), b"\xFB\x80\x80");
    test(CodePoint(0x2C_0000), b"\xFB\x80\x80\x80");
    test(End(), b"\xFF");
    test(End(), b"\xFF\x80");
    test(End(), b"\xFF\x80\x80");
    test(CodePoint(0x3C_0000), b"\xFF\x80\x80\x80");
    test(CodePoint(0x3C_6143), b"\xFF\x86\x85\x83");
}
