//! # Utils module

use std::io;

/// `Sendible` trait indicates if a type can be
/// sendible through the network as raw bytes and
/// be converted back from.
pub trait Sendible<'s>: Sized {
    /// Serialize into raw bytes 
    fn serialize(&self) -> io::Result<Vec<u8>>;

    /// Deserialize bytes back
    fn deserialize(data: &'s [u8]) -> io::Result<Self>;
}
