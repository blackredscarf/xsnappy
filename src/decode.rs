use crate::binary::uvarint;
use crate::error::{SnappyError, TAG_LITERAL, TAG_COPY1, TAG_COPY2, TAG_COPY4};
use std::ptr::{copy_nonoverlapping, copy};

/// Return the exact length of decoded data.
pub fn decode_len(src: &[u8]) -> Result<usize, SnappyError> {
    let (v, _, err) = _decode_len(src);
    return match err {
        SnappyError::None => Ok(v),
        _ => Err(err)
    }
}

/// Return `block_len` and `header_len`.
fn _decode_len(src: &[u8]) -> (usize, usize, SnappyError) {
    let (v, n) = uvarint(src);
    if n <= 0 || v > 0xffffffff {
        return (0, 0, SnappyError::Corrupt)
    }
    let word_size = 32 << (usize::MAX >> 32 & 1);
    if word_size == 32 && v > 0x7fffffff {
        return (0, 0, SnappyError::DecodeTooLarge)
    }
    return (v as usize, n as usize, SnappyError::None)
}

pub fn decode(dst: &mut [u8], src: &[u8]) -> Result<usize, SnappyError> {
    let (d_len, s, err) = _decode_len(src);
    match err {
        SnappyError::None => {}
        _ => return Err(err)
    }
    let mut p;
    if d_len <= dst.len() {
        p = &mut dst[..d_len];
    } else {
        return Err(SnappyError::DstTooSmall)
    }

    let err = _decode(p, &src[s..]);
    return match err {
        SnappyError::None => Ok(d_len),
        _ => Err(err)
    }
}

fn _decode(dst: &mut [u8], src: &[u8]) -> SnappyError {
    let (mut d, mut s, mut offset, mut length): (usize, usize, usize, usize) = (0, 0, 0, 0) ;
    while s < src.len() {
        match src[s] & 0x03 {
            TAG_LITERAL => {
                let mut x = (src[s] >> 2) as u32;
                if x < 60 {
                    s += 1;
                } else if x == 60 {
                    s += 2;
                    if s > src.len() { return SnappyError::Corrupt }
                    x = src[s-1] as u32;
                } else if x == 61 {
                    s += 3;
                    if s > src.len() { return SnappyError::Corrupt }
                    x = src[s-2] as u32 | (src[s-1] as u32) << 8;
                } else if x == 62 {
                    s += 4;
                    if s > src.len() { return SnappyError::Corrupt }
                    x = src[s-3] as u32 | (src[s-2] as u32) << 8 | (src[s-1] as u32) << 16;
                } else if x == 63 {
                    s += 5;
                    if s > src.len() { return SnappyError::Corrupt }
                    x = src[s-4] as u32 | (src[s-3] as u32) << 8
                        | (src[s-2] as u32) << 16 | (src[s-1] as u32) << 24;
                }

                length = x as usize + 1;
                if length <= 0 { return SnappyError::UnsupportedLiteralLength }
                if length > dst.len() - d || length > src.len() - s {
                    return SnappyError::Corrupt
                }
                unsafe {
                    clone(dst.as_mut_ptr().offset(d as isize),
                          src.as_ptr().offset(s as isize),
                          dst.len()-d, length);
                }
                d += length;
                s += length;
                continue
            }
            TAG_COPY1 => {
                s += 2;
                if s > src.len() { return SnappyError::Corrupt }
                length = 4 + ((src[s-2] as usize) >> 2 & 0x7);
                offset = (((src[s-2] as u32) & 0xe0) << 3 | src[s-1] as u32) as usize;
            }
            TAG_COPY2 => {
                s += 3;
                if s > src.len() { return SnappyError::Corrupt }
                length = 1 + ((src[s-3] as usize) >> 2);
                offset = (src[s-2] as u32 | (src[s-1] as u32) << 8) as usize;
            }
            TAG_COPY4 => {
                s += 5;
                if s > src.len() { return SnappyError::Corrupt }
                length = 1 + ((src[s-5] as usize) >> 2);
                offset = (src[s-4] as u32 | (src[s-3] as u32) << 8
                    | (src[s-2] as u32) << 16 | (src[s-1] as u32) << 24) as usize;
            }
            _ => {}
        }

        if offset <= 0 || d < offset || length > dst.len() - d {
            return SnappyError::Corrupt
        }

        let end = d + length;
        if offset >= 8 && length <= 16 && d + 16 <= dst.len() {
            unsafe {
                let dst_p = dst.as_mut_ptr().add(d);
                let src_p = dst.as_mut_ptr().add(d-offset);
                copy_nonoverlapping(src_p, dst_p, 8);
                copy_nonoverlapping(src_p.add(8), dst_p.add(8), 8);
            }
            d = end;
        } else if end + 24 <= dst.len() {
            unsafe {
                let mut dst_p = dst.as_mut_ptr().add(d);
                let mut src_p = dst.as_mut_ptr().add(d-offset);
                loop {
                    let diff = (dst_p as usize) - (src_p as usize);
                    if diff >= 16 { break }
                    copy(src_p, dst_p, 16);
                    d += diff;
                    dst_p = dst_p.add(diff);
                }
                while d < end {
                    copy_nonoverlapping(src_p, dst_p, 16);
                    src_p = src_p.add(16);
                    dst_p = dst_p.add(16);
                    d += 16;
                }
            }
            d = end;
        } else {
            if offset >= length {
                unsafe {
                    clone(dst.as_mut_ptr().offset(d as isize),
                          dst.as_ptr().offset((d-offset) as isize),
                          length, dst.len()-(d-offset));
                }
                d += length;
                continue
            }
            unsafe {
                forward_clone(dst.as_mut_ptr().offset(d as isize),
                              dst.as_ptr().offset((d-offset) as isize),
                              length);
            }
            d += length;
        }
    }
    if d != dst.len() {
        return SnappyError::Corrupt
    }
    SnappyError::None
}

unsafe fn clone<T: Clone>(mut dst: *mut T, src: *const T, dst_len: usize, src_len: usize) {
    let mut ptr = src;
    let mut src_end = src.offset(src_len as isize);
    let mut dst_end = dst.offset(dst_len as isize);
    while ptr != src_end && dst != dst_end {
        *dst = (*ptr).clone();
        ptr = ptr.offset(1);
        dst = dst.offset(1);
    }
}

unsafe fn forward_clone<T: Clone>(mut dst: *mut T, src: *const T, dst_len: usize) {
    let mut ptr = src;
    let mut dst_end = dst.offset(dst_len as isize);
    while dst != dst_end {
        *dst = (*ptr).clone();
        ptr = ptr.offset(1);
        dst = dst.offset(1);
    }
}