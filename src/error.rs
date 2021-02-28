use std::fmt::{Display, Formatter};
use std::fmt;

pub(crate) const TAG_LITERAL: u8 = 0x00;
pub(crate) const TAG_COPY1: u8 = 0x01;
pub(crate) const TAG_COPY2: u8 = 0x02;
pub(crate) const TAG_COPY4: u8 = 0x03;

#[derive(Debug)]
pub enum SnappyError {
    None,
    EncodeTooLarge,   // encode block is too large
    DecodeTooLarge,   // decode block is too large
    DstTooSmall,
    Corrupt,
    Unsupported,
    UnsupportedLiteralLength
}

impl Display for SnappyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", str_of_error(self))
    }
}

const NONE_ERR_MSG: &str = "snappy: none error";
const ENCODE_TOO_LARGE_ERR_MSG: &str = "snappy:  encode block is too large";
const DECODE_TOO_LARGE_ERR_MSG: &str = "snappy:  decode block is too large";
const DST_TOO_SMALL_ERR_MSG: &str = "snappy: dst len is too small";
const CORRUPT_ERR_MSG: &str = "snappy: corrupt input";
const UNSUPPORTED_ERR_MSG: &str = "snappy: unsupported input";
const UNSUPPORTED_LITERAL_LENGTH_ERR_MSG: &str = "snappy: unsupported literal length";

fn str_of_error(err: &SnappyError) -> &'static str {
    return match err {
        SnappyError::None => NONE_ERR_MSG,
        SnappyError::EncodeTooLarge => ENCODE_TOO_LARGE_ERR_MSG,
        SnappyError::DecodeTooLarge => DECODE_TOO_LARGE_ERR_MSG,
        SnappyError::DstTooSmall => DST_TOO_SMALL_ERR_MSG,
        SnappyError::Corrupt => CORRUPT_ERR_MSG,
        SnappyError::Unsupported => UNSUPPORTED_ERR_MSG,
        SnappyError::UnsupportedLiteralLength => UNSUPPORTED_LITERAL_LENGTH_ERR_MSG
    }
}
