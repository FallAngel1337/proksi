//! # Reply
//! The possible addresses to fill in the request's `rep` field

// #[allow(missing_docs)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum ReplyOpt {
    Succeeded = 0x0,
    SocksServerFailure = 0x1,
    ConnectionNotAllowed = 0x2,
    NetworkUnreachable = 0x3,
    HostUnreachable = 0x4,
    ConnectionRefused = 0x5,
    TtlExpired = 0x6,
    CommandNotSupported = 0x7,
    AddressTypeNotSupported = 0x8,
}
