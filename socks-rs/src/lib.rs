#![warn(missing_docs)]

//! # A pure Rust implementation of SOCKS 5 protocol
//! according to [`RFC 1928`](https://datatracker.ietf.org/doc/html/rfc1928)
//! TODO: Add a better description and items

/// The SOCKS protocol version
pub const SOCKS_VERSION: u8 = 0x5;

pub mod establish;
pub mod reply;
pub mod request;
pub mod utils;
