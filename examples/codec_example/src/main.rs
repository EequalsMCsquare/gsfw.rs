use bytes::{Bytes, BytesMut};
use gsfw::codec::{Codec, Encoder};

mod codec;
mod pb;
fn main() {
    let codec = codec::PBCodec;

    let cs = pb::cs_proto::Payload::Login(pb::CsLogin {
        username: String::from("eequalsmc2"),
        password: String::from("password"),
    });
    let mut buf = BytesMut::with_capacity(1024);
    let encoded_cs = codec.encoder().encode(cs, &mut buf);
}
