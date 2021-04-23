//! Tezos Node's RPC
//!
//! [rpc_api::api] defines traits and types for the api.
//!
//! Then there can be multiple implementation for those traits, like:
//! [rpc_api::http_api], which interacts with the node using **http** protocol.
//!
//! Other protocols like **WebSockets**, **GRPC + Protobuf**, etc... Can
//! be implemented using traits defined in [rpc_api::api], If the target
//! node supports chosen protocol.
//!
//!
//! Each endpoint has it's own error types. Common error is [rpc_api::api::TransportError].
//! It will contain errors, which happened because of transport issues
//! like: connection was lost, timeout, etc...

pub mod api;
pub mod http_api;
