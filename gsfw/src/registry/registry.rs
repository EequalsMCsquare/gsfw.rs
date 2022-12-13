use std::collections::HashMap;

use once_cell::sync::Lazy;

pub trait RegistryExt
where
    Self: Sized,
{
    const COUNT: usize; // message count
    const NAMES: Lazy<Vec<&'static str>>; // message names
    const IDS: Lazy<Vec<i32>>;
    const ID2NAME_MAP: Lazy<HashMap<i32, &'static str>>;
    const NAME2ID_MAP: Lazy<HashMap<&'static str, i32>>;
    const NAME_MAP: Lazy<HashMap<&'static str, Self>>;
    const ID_MAP: Lazy<HashMap<i32, Self>>;

    // frame buf layout should be [msgid][payload]
    fn decode_frame<B>(frame_buf: B) -> Result<Self, crate::error::Error>
    where
        B: bytes::Buf,
        Self: Sized;
    
    // bytes needed to encode [msgid][payload]
    fn encoded_len(&self) -> usize
    where
        Self: Sized;

    // [msgid][payload]
    fn encode_to<B>(&self, buf: &mut B) -> Result<(), crate::error::Error>
    where
        B: bytes::BufMut,
        Self: Sized;

    fn encode(&self) -> bytes::Bytes
    where
        Self: Sized;

    // [len][msgid][payload]
    fn encode_to_with_len<B>(&self, buf: &mut B) -> Result<(), crate::error::Error>
    where
        B: bytes::BufMut,
        Self: Sized;
    
    fn encode_with_len(&self) -> bytes::Bytes
    where
        Self: Sized;
}
