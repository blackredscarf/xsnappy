use std::fs::File;
use std::io::Read;

pub fn read_file_to_vec(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let mut data = vec!();
    file.read_to_end(&mut data).unwrap();
    data
}

pub fn cmp(dst: &[u8], src: &[u8]) -> bool {
    if dst.len() != src.len() {
        assert_eq!(dst.len(), src.len());
        return false
    }
    for i in 0..dst.len() {
        if dst[i] != src[i] {
            return false
        }
    }
    return true
}

#[cfg(test)]
mod tests {
    use crate::golden::{read_file_to_vec, cmp};
    use xsnappy::{decode, decode_len};
    use xsnappy::{max_encode_len, encode};

    fn test_golden_decode() {
        let src = read_file_to_vec("testdata/Mark.Twain-Tom.Sawyer.txt.rawsnappy");
        let want = read_file_to_vec("testdata/Mark.Twain-Tom.Sawyer.txt");

        let dec_len = decode_len(&src).unwrap_or(0);
        let mut dst = Vec::<u8>::with_capacity(dec_len);
        dst.resize(dec_len, 0);
        decode(&mut dst, &src);

        assert!(cmp(&dst, &want));
    }

    fn test_golden_encode() {
        let src = read_file_to_vec("testdata/Mark.Twain-Tom.Sawyer.txt");
        let want = read_file_to_vec("testdata/Mark.Twain-Tom.Sawyer.txt.rawsnappy");

        let max_len = max_encode_len(src.len());
        let mut dst = Vec::<u8>::with_capacity(max_len);
        dst.resize(max_len, 0);
        let got_len = encode(&mut dst, &src);
        dst.resize(got_len, 0);

        assert!(cmp(&dst, &want));
    }

    #[test]
    fn it_works() {
        test_golden_decode();
        test_golden_encode();
    }
}