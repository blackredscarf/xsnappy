#[cfg(test)]
mod tests {
    use xsnappy::{max_encode_len, encode};
    use xsnappy::{decode_len, decode};
    use std::str::from_utf8;

    fn compress() {
        let src = "Jingle bell, jingle bell, jingle bell rock \
                            Jingle bells swing and jingle bells ring".as_bytes();
        let mut dst = vec![0; max_encode_len(src.len())];
        let size = encode(&mut dst, src);
        dst.resize(size, 0);
        // println!("{:?}", dst);
    }

    fn decompress() {
        let src = vec![83, 52, 74, 105, 110, 103, 108, 101, 32, 98, 101, 108, 108, 44, 32, 106, 90,
                       13, 0, 20, 32, 114, 111, 99, 107, 32, 29, 43, 40, 115, 32, 115, 119, 105, 110,
                       103, 32, 97, 110, 100, 46, 53, 0, 20, 115, 32, 114, 105, 110, 103];

        let dec_len = match decode_len(&src) {
            Ok(len) => len,
            Err(err) => panic!("{}", err)
        };

        let mut dst = vec![0; dec_len];
        match decode(&mut dst, &src) {
            Ok(len) => {},
            Err(err) => panic!("{}", err)
        }
        // println!("{}", from_utf8(&dst).unwrap());
    }

    #[test]
    fn it_works() {
        compress();
        decompress();
    }
}