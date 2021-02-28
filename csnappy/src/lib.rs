
use libc::{c_int, size_t};

pub fn compress(src: &[u8], dst: &mut [u8]) -> Result<usize, String> {
    unsafe {
        let mut dst_len = snappy_max_compressed_length(src.len());
        if dst.len() < dst_len {
            return Err(format!(
                "destination buffer too small ({} < {})",
                dst.len(),
                dst_len
            ));
        }
        snappy_compress(
            src.as_ptr(),
            src.len(),
            dst.as_mut_ptr(),
            &mut dst_len,
        );
        Ok(dst_len)
    }
}

pub fn decompress(src: &[u8], dst: &mut [u8]) -> Result<usize, String> {
    unsafe {
        let mut dst_len = 0;
        snappy_uncompressed_length(
            src.as_ptr(),
            src.len() as size_t,
            &mut dst_len,
        );
        if dst.len() < dst_len {
            return Err(format!(
                "destination buffer too small ({} < {})",
                dst.len(),
                dst_len
            ));
        }
        let r = snappy_uncompress(
            src.as_ptr(),
            src.len(),
            dst.as_mut_ptr(),
            &mut dst_len,
        );
        if r == 0 {
            Ok(dst_len)
        } else {
            Err("snappy: invalid input".to_owned())
        }
    }
}

pub fn max_encode_len(src_len: usize) -> usize {
    unsafe { snappy_max_compressed_length(src_len) }
}

pub fn decode_len(src: &[u8]) -> usize {
    let mut dst_len = 0;
    unsafe { snappy_uncompressed_length(src.as_ptr(),
                                        src.len() as size_t,
                                        &mut dst_len) };
    dst_len
}

extern "C" {
    fn snappy_compress(
        input: *const u8,
        input_len: size_t,
        compressed: *mut u8,
        compressed_len: *mut size_t,
    ) -> c_int;

    fn snappy_uncompress(
        compressed: *const u8,
        compressed_len: size_t,
        uncompressed: *mut u8,
        uncompressed_len: *mut size_t,
    ) -> c_int;

    fn snappy_max_compressed_length(input_len: size_t) -> size_t;

    fn snappy_uncompressed_length(
        compressed: *const u8,
        compressed_len: size_t,
        result: *mut size_t,
    ) -> c_int;
}


#[cfg(test)]
mod tests {
    use crate::{max_encode_len, decode_len, compress, decompress};
    use std::cmp::max;

    #[test]
    fn it_works() {
        let mut vs = vec![];
        for i in 0..150 {
            vs.push(i);
        }

        let max_len = max_encode_len(vs.len());
        let mut dst = Vec::<u8>::with_capacity(max_len);
        dst.resize(max_len, 0);
        let enc_len = compress(&vs, &mut dst).unwrap();
        dst.resize(enc_len, 0);

        let dec_len = decode_len(&dst);
        let mut res = Vec::<u8>::with_capacity(dec_len);
        res.resize(dec_len, 0);
        decompress(&dst, &mut res);

        assert_eq!(vs.len(), res.len());
        for i in 0..res.len() {
            assert_eq!(vs[i], res[i]);
        }
    }
}
