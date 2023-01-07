//! # Utils module

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{net::TcpStream};
use std::io;

/// To avoid implementating the same (de)serialization
/// methods every single time for any new time it's easy to just
/// implement a trait.
pub trait Sendible<'s>: Sized {
    fn serialize(&self) -> Option<Vec<u8>>;

    fn deserialize(data: &'s [u8]) -> Option<Self>;
}

#[derive(Debug)]
pub struct Delivery<'a, RW = TcpStream>
where
    RW: AsyncReadExt + AsyncWriteExt + Send + Unpin,
{
    stream: &'a mut RW,
}

#[allow(unused)]
impl<'a, RW> Delivery<'a, RW>
where
    RW: AsyncReadExt + AsyncWriteExt + Send + Unpin,
{
    pub fn new(stream: &'a mut RW) -> Self {
        Self { stream }
    }

    pub async fn send<'s, S>(&mut self, data: S) -> io::Result<()>
    where
        S: Sendible<'s> + Send,
    {
        self.send_raw(&Sendible::serialize(&data).unwrap()).await
    }

    pub async fn recv<'r, S>(&mut self) -> io::Result<S>
    where
        S: Sendible<'r> + Send,
    {
        // Maybe a workaround but it works
        let data = Box::leak(Box::new(self.recv_raw().await.unwrap()));
        Ok(<S as Sendible>::deserialize(data).unwrap())
    }

    #[inline]
    pub async fn send_raw(&mut self, data: &[u8]) -> io::Result<()> {
        self.stream.write_all(data).await
    }

    pub async fn recv_raw(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::<u8>::with_capacity(1024);

        if self.stream.read_buf(&mut buf).await.unwrap() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Connection closed!",
            ));
        }

        Ok(buf)
    }

    pub fn get_ref(&self) -> &RW {
        &*self.stream
    }

    pub fn get_ref_mut(&mut self) -> &mut RW {
        self.stream
    }
}
