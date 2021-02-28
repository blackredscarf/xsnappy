/*!
This is a Snappy compression algorithm library written in Rust.
It is ported from [Go implementation](https://github.com/golang/snappy) with similar interface.

Provide basic interfaces:
- `max_encode_len(src_len)`: Get the max length of encoded data.
- `encode(dst, src)`: Encode `src` to `dst`.
- `decode_len(src)`: Get the exact length of decoded data.
- `decode_len(dst, src)`: Decode `src` to `dst`.

# Examples:

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

*/

mod binary;
mod encode;
mod decode;
pub mod error;
use error::SnappyError;


/// Encode `src` to `dst`. The `dst` must be initialized with a certain length.
/// Return the exact length of encoded data.
/// # Examples:
///
/// ```rust
/// use xsnappy::{max_encode_len, encode};
///
/// let mut src = b"hello world! hello world!";
/// let mut dst = vec![0; max_encode_len(src.len())];
/// let len = encode(&mut dst, src);
/// dst.resize(len, 0); // resize to exact length
/// ```
/// # Panics:
/// Panics if the length of `dst` is less then `max_encode_len(src.len())`
pub fn encode(dst: &mut [u8], src: &[u8]) -> usize {
    encode::encode(dst, src)
}

/// Decode `src` to `dst`. The `dst` must be initialized with a certain length.
/// Return the exact length of decoded data.
/// # Examples:
///
/// ```rust
/// use xsnappy::{decode_len, decode};
///
/// let dec_len = decode_len(src).unwrap();
/// let mut dst = vec![0; dec_len];
/// decode(&mut dst, src);
/// ```
pub fn decode(dst: &mut [u8], src: &[u8]) -> Result<usize, SnappyError> {
    decode::decode(dst, src)
}

/// Return the max length of encoded data
pub fn max_encode_len(src_len: usize) -> usize {
    encode::max_encode_len(src_len)
}

/// Return the exact length of decoded data.
pub fn decode_len(src: &[u8]) -> Result<usize, SnappyError> {
    decode::decode_len(src)
}