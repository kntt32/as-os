#[allow(unused)]
pub fn as_utf16<const L: usize>(s: &str) -> [u16; L] {
    let mut buff = [0u16; L];
    let mut buff_len: usize = 0;

    let mut s_utf16 = s.encode_utf16();

    while buff_len < L - 1 {
        let Some(utf16_code) = s_utf16.next() else {
            break;
        };
        buff[buff_len] = utf16_code;
        buff_len += 1;
    }

    buff
}
