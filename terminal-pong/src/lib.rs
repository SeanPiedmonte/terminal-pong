pub mod ai;
pub mod client;
pub mod eventlog;
pub mod net;
pub mod physics;
pub mod render;
pub mod server;
pub mod solo;

/// Protobuf types generated from `proto/message.proto` by `build.rs`.
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}
