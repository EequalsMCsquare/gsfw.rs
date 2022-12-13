pub mod chanrpc;
pub mod codec;
pub mod component;
pub mod error;
pub mod gs;
pub mod network;
pub mod registry;
#[cfg(feature = "derive")]
pub use registry::{Protocol, RegistryExt};
#[cfg(feature = "derive")]
pub use gsfw_derive::{Protocol, Registry};

#[cfg(feature = "util")]
pub mod util {
    pub use gsfw_util::*;
    #[cfg(feature = "derive")]
    pub use gsfw_derive::Dirty;
}