mod example;
mod golden;
mod bench;
use criterion::{criterion_main, criterion_group};

criterion_group!(benches, bench::run_all_benches);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use bytes::{BytesMut, BufMut};
    use xsnappy::{encode, max_encode_len};
    use xsnappy::{decode_len, decode};
    use xsnappy::error::SnappyError;
    use std::hint::unreachable_unchecked;
    use crate::golden::read_file_to_vec;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn print_bytes(bs: &[u8], start: usize, end: usize) {
        let mut i = start;
        while i < end {
            print!("{}, ", bs[i]);
            i += 1;
        }
        println!()
    }

    fn custom_curve(len: usize) -> Vec<u32> {
        let mut vs = vec![];
        let mut i = 0;
        while i < len {
            let mut v = 0.0;
            if i < 1000 {
                v = f64::floor(f64::abs(f64::sin(std::f64::consts::PI / 20.0 * i as f64)) * 1000.0)
            } else {
                v = f64::floor(f64::abs(f64::cos(std::f64::consts::PI / 20.0 * i as f64)) * 1000.0)
            }
            vs.push(v as u32);
            i += 1;
        }
        vs
    }

    fn test_encode(src: &[u8], get_len: &mut usize) -> BytesMut {
        let len = max_encode_len(src.len());
        let mut dst = BytesMut::new();
        dst.resize(len, 0);
        *get_len = encode(&mut dst[..], &src);
        return dst
    }

    fn test_short_encode() {
        let mut src = BytesMut::from("hello");
        let mut len = 0 as usize;
        let mut dst = test_encode(&src, &mut len);
        assert_eq!(len, 7);
        assert_eq!(dst[0], 5);
        assert_eq!(dst[1], 16);
        assert_eq!(dst[2], 104);
        assert_eq!(dst[3], 101);
        assert_eq!(dst[4], 108);
        assert_eq!(dst[len-1], 111);
    }

    fn test_mid_encode() {
        let mut src = BytesMut::from("You know some birds are not meant to be caged, \
                                        their feathers are just too bright.");
        let mut len = 0 as usize;
        let mut dst = test_encode(&src, &mut len);
        assert_eq!(len, 83);
        assert_eq!(dst[0], 82);
        assert_eq!(dst[31], 109);
        assert_eq!(dst[63], 115);
        assert_eq!(dst[94], 0);
        assert_eq!(dst[126], 0);
        assert_eq!(dst[len-1], 46);
        assert_eq!(dst.len(), 127);
    }

    fn test_empty_encode() {
        let mut src = BytesMut::from("");
        let mut len = 0 as usize;
        let mut dst = test_encode(&src, &mut len);
        assert_eq!(len, 1);
        assert_eq!(dst[0], 0);
        assert_eq!(dst[1], 0);
        assert_eq!(dst.len(), 32);
    }

    fn test_long_encode() {
        let mut src = BytesMut::from("");
        let bs_len = 65536 as usize;
        let vs = custom_curve(bs_len / 4);
        for i in vs.iter() {
            src.put_u32_le(*i)
        }
        let mut len = 0 as usize;   // real length
        let mut dst = test_encode(&src, &mut len);
        assert_eq!(dst.len(), 76490);
        assert_eq!(len, 3161);
        assert_eq!(dst[0], 128);
        assert_eq!(dst[10], 0);
        assert_eq!(dst[40], 219);
        assert_eq!(dst[2000], 254);
        assert_eq!(dst[3160], 15);
    }

    fn test_long_long_encode() {
        let mut src = BytesMut::from("");
        let bs_len = 65537 * 5 as usize;
        let vs = custom_curve(bs_len / 4);
        for i in vs.iter() {
            src.put_u32_le(*i)
        }

        let mut len = 0 as usize;
        let mut dst = test_encode(&src, &mut len);
        assert_eq!(dst.len(), 382330);
        assert_eq!(len, 15738);
        assert_eq!(dst[0], 132);
        assert_eq!(dst[100], 0);
        assert_eq!(dst[1000], 15);
        assert_eq!(dst[2000], 254);
        assert_eq!(dst[10001], 80);
        assert_eq!(dst[15737], 0);
    }

    fn test_decode(src: &[u8]) -> BytesMut {
        let de_len = match decode_len(&src[..]) {
            Ok(len) => len,
            _ => 0
        };
        let mut de_dst = BytesMut::new();
        de_dst.resize(de_len, 0);
        decode(&mut de_dst[..], src);
        de_dst
    }

    fn test_short_decode() {
        let mut src = BytesMut::from("hello");
        let mut real_encode_len = 0 as usize;
        let mut en_dst = test_encode(&src, &mut real_encode_len);
        let mut dst = test_decode(&en_dst[..]);
        assert_eq!(dst.len(), 5);
        let ans: Vec<u8> = vec![104, 101, 108, 108, 111];
        for (i, v) in dst.iter().enumerate() {
            assert_eq!(*v, ans[i]);
        }
    }

    fn test_mid_decode() {
        let mut src = BytesMut::from("You know some birds are not meant to be caged, \
                                        their feathers are just too bright.");
        let mut real_encode_len = 0 as usize;
        let mut en_dst = test_encode(&src, &mut real_encode_len);
        let mut dst = test_decode(&en_dst[..]);
        assert_eq!(dst.len(), 82);
        assert_eq!(dst[0], 89);
        assert_eq!(dst[dst.len()-1], 46);
        assert_eq!(dst[dst.len()-10], 111);
        assert_eq!(dst[dst.len()/2], 97);
        assert_eq!(dst[dst.len()/3], 32);
        assert_eq!(dst[dst.len()/4], 97);
    }

    fn test_long_decode() {
        let mut src = BytesMut::from("");
        let bs_len = 65537 * 5 as usize;
        let vs = custom_curve(bs_len / 4);
        for i in vs.iter() {
            src.put_u32_le(*i)
        }

        let mut real_encode_len = 0 as usize;
        let mut en_dst = test_encode(&src, &mut real_encode_len);
        let mut dst = test_decode(&en_dst[..]);
        assert_eq!(dst.len(), 327684);
        assert_eq!(dst[0], 0);
        assert_eq!(dst[dst.len()-dst.len()/3], 75);
        assert_eq!(dst[dst.len()-dst.len()/4], 0);
        assert_eq!(dst[dst.len()/2], 0);
        assert_eq!(dst[dst.len()/3], 197);
        assert_eq!(dst[dst.len()/4], 3);
    }

    fn test_corrupt() {
        let mut src = BytesMut::from("You know some birds are not meant to be caged, \
                                        their feathers are just too bright.");
        let mut real_encode_len = 0 as usize;
        let mut en_dst = test_encode(&src, &mut real_encode_len);

        en_dst.resize(en_dst.len() - 20, 0);
        let de_len = match decode_len(&en_dst[..]) {
            Ok(len) => len,
            _ => 0,
        };

        let mut de_dst = BytesMut::new();
        de_dst.resize(de_len, 0);
        match decode(&mut de_dst[..], &en_dst[..]) {
            Ok(len) => unreachable!(),
            Err(e) => {
                match e {
                    SnappyError::Corrupt => assert!(true),
                    _ => assert!(false)
                }
            }
        }
    }

    #[test]
    fn it_works() {
        test_short_encode();
        test_mid_encode();
        test_empty_encode();
        test_long_encode();
        test_long_long_encode();
        test_short_decode();
        test_mid_decode();
        test_long_decode();
        test_corrupt();
    }
}