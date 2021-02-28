use crate::binary::{put_uvarint, load32, load64};
use crate::error::{TAG_LITERAL, TAG_COPY2, TAG_COPY1};

const MAX_BLOCK_SIZE: usize = 65536;
const INPUT_MARGIN: usize = 16 - 1;
const MIN_NON_LITERAL_BLOCK_SIZE: usize = 1 + 1 + INPUT_MARGIN;

/// Return the max length of encoded data
pub fn max_encode_len(src_len: usize) -> usize {
    let mut n = src_len;
    if n > 0xffffffff {
        return 0
    }
    n = 32 + n + n / 6;
    if n > 0xffffffff {
        return 0
    }
    return n
}

pub fn encode(dst: &mut [u8], src: &[u8]) -> usize {
    let n = max_encode_len(src.len());
    if n == 0 {
        panic!("snappy: encode block is too large")
    } else if dst.len() < n as usize {
        panic!("snappy: dst len is too small")
    }
    let mut d = put_uvarint(dst, src.len() as u64);
    let mut p = &src[..];
    while p.len() > 0 {
        if p.len() < MIN_NON_LITERAL_BLOCK_SIZE {
            d += emit_literal(&mut dst[d..],  p);
            p = &[];
        } else {
            if p.len() > MAX_BLOCK_SIZE {
                d += encode_block(&mut dst[d..], &p[..MAX_BLOCK_SIZE]);
                p = &p[MAX_BLOCK_SIZE..];
            } else {
                d += encode_block(&mut dst[d..], p);
                p = &[];
            }
        }
    }
    return d
}

/// Emit a literal from `lit` to `dst`.
fn emit_literal(dst: &mut [u8], lit: &[u8]) -> usize {
    let mut i = 0;
    let n = lit.len() - 1;
    if n < 60 {
        dst[0] = (n as u8) << 2 | TAG_LITERAL;
        i = 1;
    } else if n < 1 << 8 {
        dst[0] = 60 << 2 | TAG_LITERAL;
        dst[1] = n as u8;
        i = 2;
    } else {
        dst[0] = 61 << 2 | TAG_LITERAL;
        dst[1] = n as u8;
        dst[2] = (n >> 8) as u8;
        i = 3;
    }
    let mut j = 0;
    while j < lit.len() {
        dst[i] = lit[j];
        i += 1;
        j += 1;
    }
    return i
}

fn encode_block(dst: &mut [u8], src: &[u8]) -> usize {
    const MAX_TABLE_SIZE: usize = 1 << 14;
    const TABLE_MASK: usize = MAX_TABLE_SIZE - 1;

    let mut shift = (32 - 8) as u32;
    let mut table_size: usize = 1 << 8;
    while table_size < MAX_TABLE_SIZE && table_size < src.len() {
        shift -= 1;
        table_size *= 2;
    }
    let mut table: [u16; MAX_TABLE_SIZE] = [0; MAX_TABLE_SIZE];
    let s_limit = src.len() - INPUT_MARGIN;
    let mut next_emit = 0;

    let mut d = 0;
    let mut s = 1;
    let mut next_hash = hash(load32(src, s), shift) as usize;

    loop {
        let mut skip = 32;
        let mut next_s = s;
        let mut candidate = 0;
        loop {
            s = next_s;
            let bytes_between_hash_lookups = skip >> 5;
            next_s = s + bytes_between_hash_lookups;
            skip += bytes_between_hash_lookups;
            if next_s > s_limit {
                return emit_remainder(dst, src, next_emit, d);
            }
            candidate = table[next_hash&TABLE_MASK] as usize;
            table[next_hash&TABLE_MASK] = s as u16;
            next_hash = hash(load32(src, next_s), shift) as usize;
            if load32(src, s) == load32(src, candidate) {
                break
            }
        }

        d += emit_literal(&mut dst[d..], &src[next_emit..s]);

        loop {
            let mut base = s;
            s += 4;
            let mut i = candidate + 4;

            let mut advance = true;
            while s + 8 < src.len() {
                let x = load64(src, s);
                let y = load64(src, i);
                if x == y {
                    s += 8;
                    i += 8;
                } else {
                    // x and y are probably just partly equal.
                    // Get the equal part of x and y.
                    let z = x.to_le() ^ y.to_le();
                    s += z.trailing_zeros() as usize / 8;
                    advance = false;
                    break;
                }
            }
            if advance {
                while s < src.len() && src[i] == src[s] {
                    i = i + 1;
                    s = s + 1;
                }
            }

            d += emit_copy(&mut dst[d..], base-candidate, s-base);
            next_emit = s;
            if s >= s_limit {
                return emit_remainder(dst, src, next_emit, d)
            }

            let x = load64(src, s-1);
            let prev_hash = hash((x>>0) as u32, shift) as usize;
            table[prev_hash&TABLE_MASK] = (s - 1) as u16;
            let curr_hash = hash((x>>8) as u32, shift) as usize;
            candidate = table[curr_hash&TABLE_MASK] as usize;
            table[curr_hash&TABLE_MASK] = s as u16;
            if (x>>8) as u32 != load32(src, candidate) {
                next_hash = hash((x>>16) as u32, shift) as usize;
                s += 1;
                break
            }
        }
    }
}

fn extend_match(src: &[u8], mut s: usize, mut i: usize) {
    while s + 8 < src.len() {
        let x = load64(src, s);
        let y = load64(src, i);
        if x == y {
            s += 8;
            i += 8;
        } else {
            let z = x.to_le() ^ y.to_le();
            s += z.trailing_zeros() as usize / 8;
        }
    }
    while s < src.len() && src[i] == src[s] {
        i = i + 1;
        s = s + 1;
    }
}

/// Emit remainder data from `src` to `dst`.
fn emit_remainder(dst: &mut [u8], src: &[u8], next_emit: usize, mut d: usize) -> usize {
    if next_emit < src.len() {
        d += emit_literal(&mut dst[d..], &src[next_emit..]);
    }
    d
}

/// Encode `offset` and `length` to `dst`.
fn emit_copy(dst: &mut [u8], offset: usize, mut length: usize) -> usize {
    let mut i = 0;
    while length >= 68 {
        dst[i+0] = 63 << 2 | TAG_COPY2;
        dst[i+1] = offset as u8;
        dst[i+2] = (offset >> 8) as u8;
        i += 3;
        length -= 64;
    }
    if length > 64 {
        dst[i+0] = (59 << 2) | TAG_COPY2;
        dst[i+1] = offset as u8;
        dst[i+2] = (offset >> 8) as u8;
        i += 3;
        length -= 60;
    }
    if length >= 12 || offset >= 2048 {
        dst[i+0] = ((length-1) as u8) << 2 | TAG_COPY2;
        dst[i+1] = offset as u8;
        dst[i+2] = (offset >> 8) as u8;
        return i + 3
    }
    dst[i+0] = ((offset>>8) as u8) << 5 | ((length-4) as u8) << 2 | TAG_COPY1;
    dst[i+1] = offset as u8;
    return i + 2
}

fn hash(u: u32, shift: u32) -> u32 {
    ((u as u64 * 0x1e35a7bd) as u32 >> shift) as u32
}