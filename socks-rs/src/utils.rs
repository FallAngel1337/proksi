//! # Utils module

use std::io;

/// To avoid implementating the same (de)serialization
/// methods every single time for any new time it's easy to just
/// implement a trait.
pub trait Sendible<'s>: Sized {
    fn serialize(&self) -> io::Result<Vec<u8>>;

    fn deserialize(data: &'s [u8]) -> io::Result<Self>;
}
