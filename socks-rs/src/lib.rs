#![warn(missing_docs)]

//! # A pure Rust implementation of SOCKS 5 protocol
//! According to [`RFC 1928`](https://datatracker.ietf.org/doc/html/rfc1928)

/// The SOCKS protocol version
pub const SOCKS_VERSION: u8 = 0x5;

pub mod auth;
pub mod establish;
pub mod reply;
pub mod request;

/// `Sendible` trait indicates if a type can be
/// sendible through the network as raw bytes and
/// be converted back from.
pub trait Sendible<'s>: Sized {
    /// Serialize into raw bytes
    fn serialize(&self) -> std::io::Result<Vec<u8>>;

    /// Deserialize bytes back
    fn deserialize(data: &'s [u8]) -> std::io::Result<Self>;
}
