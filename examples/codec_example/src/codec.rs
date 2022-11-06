use bytes::{Buf, BufMut, Bytes};
use prost::{self, Message};

use crate::pb;
pub struct PBCodec;
pub struct PBEncoder;
pub struct PBDecoder;

impl gsfw::codec::Encoder<pb::sc_proto::Payload> for PBEncoder {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        item: pb::sc_proto::Payload,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        dst.put_u32(item.encoded_len() as u32);
        item.encode(dst);
        Ok(())
    }
}

impl gsfw::codec::Decoder for PBDecoder {
    type Item = pb::cs_proto::Payload;

    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut buf = Bytes::from(src.to_vec());
        if buf.len() < 4 {
            return Ok(None);
        }
        let len = buf.get_u32() as usize;
        if buf.len() < 4 + len {
            return Ok(None);
        }
        src.advance(4 + len);
        Ok(pb::CsProto::decode(buf.take(len).into_inner())
            .unwrap()
            .payload)
    }
}

impl gsfw::codec::Codec for PBCodec {
    type EncodeFrom = pb::sc_proto::Payload;

    type DecodeTo = pb::cs_proto::Payload;

    type Error = anyhow::Error;

    type Decoder = PBDecoder;

    type Encoder = PBEncoder;

    fn encoder(&self) -> Self::Encoder {
        PBEncoder
    }

    fn decoder(&self) -> Self::Decoder {
        PBDecoder
    }
}
