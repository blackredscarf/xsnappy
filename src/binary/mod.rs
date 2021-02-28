
pub fn put_uvarint(buf: &mut [u8], mut x: u64) -> usize {
    let mut i = 0;
    while x >= 0x80 {
        buf[i] = x as u8 | 0x80;
        x >>= 7;
        i += 1;
    }
    buf[i] = x as u8;
    (i + 1) as usize
}

pub fn uvarint(buf: &[u8]) -> (u64, isize) {
    let mut x: u64 = 0;
    let mut s: u32 = 0;
    for i in 0..buf.len() {
        let b = buf[i];
        if b < 0x80 {
            if i > 9 || i == 9 && b > 1 {
                return (0, -1 * (i + 1) as isize)
            }
            return (x | ((b as u64) << s) as u64, (i + 1) as isize)
        }
        x = x | (((b & 0x7f) as u64) << s as u64);
        s += 7;
    }
    return (0, 0)
}

pub fn load32(b: &[u8], i: usize) -> u32 {
    let b2 = &b[i..i+4];
    return b2[0] as u32 | (b2[1] as u32) << 8 | (b2[2] as u32) << 16 | (b2[3] as u32) << 24
}

pub fn load64(b: &[u8], i: usize) -> u64 {
    let b2 = &b[i..i+8];
    return b2[0] as u64 | (b2[1] as u64) << 8 | (b2[2] as u64) << 16 | (b2[3] as u64) << 24 |
        (b2[4] as u64) << 32 | (b2[5] as u64) << 40 | (b2[6] as u64) << 48 | (b2[7] as u64) << 56
}