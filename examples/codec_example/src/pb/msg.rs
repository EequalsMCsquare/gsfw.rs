#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CsProto {
    #[prost(oneof = "cs_proto::Payload", tags = "1, 2")]
    pub payload: ::core::option::Option<cs_proto::Payload>,
}
/// Nested message and enum types in `CsProto`.
pub mod cs_proto {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "1")]
        Login(super::CsLogin),
        #[prost(message, tag = "2")]
        Echo(super::CsEcho),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScProto {
    #[prost(oneof = "sc_proto::Payload", tags = "1, 2")]
    pub payload: ::core::option::Option<sc_proto::Payload>,
}
/// Nested message and enum types in `ScProto`.
pub mod sc_proto {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Payload {
        #[prost(message, tag = "1")]
        Login(super::ScLogin),
        #[prost(message, tag = "2")]
        Echo(super::ScEcho),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CsLogin {
    #[prost(string, tag = "1")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub password: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScLogin {
    #[prost(uint32, tag = "1")]
    pub code: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CsEcho {
    #[prost(string, tag = "1")]
    pub content: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScEcho {
    #[prost(string, tag = "1")]
    pub reply: ::prost::alloc::string::String,
}
