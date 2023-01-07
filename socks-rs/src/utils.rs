//! # Utils module

use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::{sync::Arc, net::SocketAddr};
use tokio::sync::Mutex;
use std::io;

/// To avoid implementating the same (de)serialization
/// methods every single time for any new time it's easy to just
/// implement a trait.
pub trait Sendible<'s>: Sized {
    fn serialize(&self) -> Option<Vec<u8>>;

    fn deserialize(data: &'s [u8]) -> Option<Self>;
}

/// Wrapper around a TcpStream
#[derive(Debug)]
pub struct Delivery {
    stream: Arc<Mutex<TcpStream>>,
}

#[allow(unused)]
impl Delivery {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub async fn send<'s, S>(&self, data: S) -> io::Result<()>
    where S: Sendible<'s> + Send
    {
        self.send_raw(&Sendible::serialize(&data).unwrap()).await?;
        Ok(())
    }

    pub async fn recv<'r, R>(&self) -> io::Result<R>
    where R: Sendible<'r> + Send
    {
        // Maybe a workaround but it works
        let data = Box::leak(Box::new(self.recv_raw().await.unwrap()));
        Ok(
            <R as Sendible>::deserialize(data).unwrap()
        )
    }

    pub async fn send_raw(&self, data: &[u8]) -> io::Result<()> {
        let mut stream = self.stream
            .lock()
            .await;

        stream
            .flush()
            .await?;

       stream
            .write_all(data)
            .await?;
        Ok(())
    }

    pub async fn recv_raw(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::<u8>::with_capacity(1024);

        let mut stream = self.stream
            .lock()
            .await;

        stream
            .flush()
            .await
            .unwrap();

        if stream
            .read_buf(&mut buf)
            .await
            .unwrap() == 0
        {
            return Err(
                io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed!")
            )
        }
        
        Ok(buf)
    }

    pub async fn address(&self) -> io::Result<SocketAddr> {
        self.stream.lock().await.peer_addr()
    }

    pub fn stream(&self) -> Arc<Mutex<TcpStream>> {
        self.stream.clone()
    }
}