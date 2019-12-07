// Everything below was copied from the standard library, since
// [next_code_point()] is not exposed:
// https://github.com/rust-lang/rust/blob/4560ea788cb760f0a34127156c78e2552949f734/src/libcore/str/mod.rs#L500

const CONT_MASK: u8 = 0b0011_1111;

#[inline]
fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    u32::from(byte & (0x7F >> width))
}

#[inline]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | u32::from(byte & CONT_MASK)
}

#[inline]
fn unwrap_or_0(opt: Option<&u8>) -> u8 {
    match opt {
        Some(&byte) => byte,
        None => 0,
    }
}

#[inline]
pub fn next_code_point<'a, I: Iterator<Item = &'a u8>>(
    bytes: &mut I,
) -> Option<u32> {
    let x = *bytes.next()?;
    if x < 128 {
        return Some(u32::from(x));
    }

    let init = utf8_first_byte(x, 2);
    let y = unwrap_or_0(bytes.next());
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        let z = unwrap_or_0(bytes.next());
        let y_z = utf8_acc_cont_byte(u32::from(y & CONT_MASK), z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            let w = unwrap_or_0(bytes.next());
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    Some(ch)
}
