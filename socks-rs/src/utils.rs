//! # Utils module

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::io;

/// To avoid implementating the same (de)serialization
/// methods every single time for any new time it's easy to just
/// implement a trait.
pub trait Sendible<'s>: serde::Serialize + serde::Deserialize<'s> {
    fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serialize(self)
            .map_or_else(
                |e| { eprintln!("Could not serialize the request! {e:?}"); None },
                Some
            )
    }

    fn deserialize(data: &'s [u8]) -> Option<Self> {
        bincode::deserialize(data)
            .map_or_else(
                |e| { eprintln!("Could not deserialize the request! {e:?}"); None },
                Some
            )
    }
}

/// Wrapper around a TcpStream
#[derive(Debug)]
pub struct Delivery {
    stream: Arc<Mutex<TcpStream>>
}

impl Delivery {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub async fn send<'s, S>(&self, data: S) -> io::Result<()>
    where S: Sendible<'s> + Send
    {
        self.stream
            .lock()
            .await
            .write_all(&Sendible::serialize(&data)
            .unwrap())
            .await?;
        Ok(())
    }
    
    // TODO: find a way that don't need to borrow a Vec<u8>
    pub async fn recv<'r, R>(&self, buf: &'r mut Vec<u8>) -> io::Result<R>
    where R: Sendible<'r> + Send
    {
        if self.stream
            .lock()
            .await
            .read_buf(buf)
            .await
            .unwrap() == 0
        {
            return Err(
                io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed!")
            )
        }
        
        Ok(
            <R as Sendible>::deserialize(buf).unwrap()
        )
    }
}