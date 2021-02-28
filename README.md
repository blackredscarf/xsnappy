# xSnappy
This is a Snappy compression algorithm library written in Rust. It is ported from [Go implementation](https://github.com/golang/snappy) with similar interface. 

## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
xsnappy="0.1.0"
```
Compress:
```rust
use xsnappy::{max_encode_len, encode};

let src = "Jingle bell, jingle bell, jingle bell rock \
                    Jingle bells swing and jingle bells ring".as_bytes();
let mut dst = vec![0; max_encode_len(src.len())];
let size = encode(&mut dst, src);
dst.resize(size, 0);
println!("{:?}", dst);
```
Decompress:
```rust
use xsnappy::{decode_len, decode};
use std::str::from_utf8;

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
println!("{}", from_utf8(&dst).unwrap());
```

## Benchmark
Benchmarks were run on an Intel i7-8700K.
```
group                             cpp                  xsnappy
----------------------------------------------------------------------
compress/zflat00_html           189.189 M/s           860.49 M/s
compress/zflat01_urls           102.937 M/s           461.47 M/s
compress/zflat02_jpg            6.79115 G/s           1.8137 G/s
compress/zflat03_jpg_200        86.8056 M/s           185.71 M/s
compress/zflat04_pdf            1.6572 G/s            1.8927 G/s
compress/zflat05_html4          186.047 M/s           821.19 M/s
compress/zflat06_txt1           76.2514 M/s           345.15 M/s
compress/zflat07_txt2           73.0001 M/s           322.59 M/s
compress/zflat08_txt3           78.9303 M/s           360.85 M/s
compress/zflat09_txt4           67.2239 M/s           300.02 M/s
compress/zflat10_pb             231.617 M/s           1.0240 G/s
compress/zflat11_gaviota        105 M/s               559.65 M/s

decompress/uflat00_html         542.587 M/s           1.4985 G/s
decompress/uflat01_urls         347.698 M/s           892.58 M/s
decompress/uflat02_jpg          24.3476 G/s           17.148 G/s
decompress/uflat03_jpg_200      333.46 M/s            912.77 M/s
decompress/uflat04_pdf          4.78034 G/s           12.241 G/s
decompress/uflat05_html4        509.091 M/s           1.3875 G/s
decompress/uflat06_txt1         193.427 M/s           509.50 M/s
decompress/uflat06_txt2         176.779 M/s           488.38 M/s
decompress/uflat06_txt3         204.232 M/s           570.69 M/s
decompress/uflat06_txt4         165.199 M/s           445.71 M/s
decompress/uflat10_pb           766.616 M/s           2.0153 G/s
decompress/uflat11_gaviota      229.091 M/s           841.87 M/s
```

## References
- [google/snappy](https://github.com/google/snappy)
- [golang/snappy](https://github.com/golang/snappy)
- [BurntSushi/rust-snappy](https://github.com/BurntSushi/rust-snappy)
