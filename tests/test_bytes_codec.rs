use bytes::{BufMut, Buf};
use gsfw::codec;

/*
    FORMAT:
        [length: 4 bytes][msgid: 2 bytes][payload: ...]
*/

enum Msg {
    CsEcho { content: String },
    ScEcho { code: u32, reply: String },
}

struct CodecA;

struct EncoderA;

struct DecoderA;

impl tokio_util::codec::Encoder<Msg> for EncoderA {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Msg, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        match item {
            Msg::ScEcho { code, reply } => {
                let payload_length = std::mem::size_of::<u32>() + reply.len();
                dst.put_u32(payload_length as u32);
                dst.put_u16(2);
                dst.put_u32(code);
                dst.put_slice(reply.as_bytes());
                return Ok(());
            }
            _ => todo!(),
        };
    }
}

impl tokio_util::codec::Decoder for DecoderA {
    type Item = Msg;

    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let payload_len = src.get_u32();
        let msg_id = src.get_u16();
        match msg_id {
            1 => {
            }
            _ => todo!()
        }
    }
}
