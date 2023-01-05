//! # Server
//! SOCKS Server releated module
//! TODO: Add a better description

use std::net::{ToSocketAddrs, SocketAddr};
use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use crate::{
    establish::{Methods, EstablishRequest, EstablishResponse},
    SOCKS_VERSION, Sendible
};

/// The `Server` struct that holds the
/// SOCKS Server configuration
#[derive(Debug, Clone)]
pub struct Server {
    version: u8,
    supported_methods: Vec<Methods>,
    addr: SocketAddr,
}

impl Server {
    /// Constructs a new Server
    pub fn new<S>(addr: S, supported_methods: &[Methods]) -> io::Result<Self>
    where
        S: ToSocketAddrs
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        Ok(
            Self {
                version: SOCKS_VERSION,
                supported_methods: supported_methods.to_vec(),
                addr
            }
        )
    }

    /// Start the server and listen for new connections
    pub async fn start(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        let supported_methods = Arc::new(self.supported_methods);

        loop {
            let (mut stream, _) = listener.accept().await?;
            let methods = supported_methods.clone();
            
            tokio::spawn(async move {
                Self::establish(&mut stream, methods.clone()).await.unwrap();
            });
        }
    }

    async fn establish(stream: &mut TcpStream, methods: Arc<Vec<Methods>>) -> io::Result<()> {
        let estbl_req = recv::<EstablishRequest>(stream, &mut Vec::with_capacity(100)).await?;
                
        if methods.iter().any(
            |x| estbl_req.methods().contains(x)
        ) {
            if let Some(&val) = methods.iter().max_by_key(|&&x| x as u8) {
                send(stream, EstablishResponse::new(val)).await?;
            }
        } else {
            send(stream, EstablishResponse::new(Methods::NoAcceptableMethods)).await?;
        }

        Ok(())
    }
}

async fn send<'s, S: Sendible<'s>> (stream: &mut TcpStream, data: S) -> io::Result<()> {
    stream.write_all(&Sendible::serialize(&data).unwrap()).await?;
    Ok(())
}

// TODO: find a way that don't nee to borrow a Vec<u8>
async fn recv<'s, S: Sendible<'s>>(stream: &mut TcpStream, buf: &'s mut Vec<u8>) -> io::Result<S> {
    if stream.read_buf(buf).await? == 0 {
        return Err(
            io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed!")
        )
    }
    
    Ok(
        <S as Sendible>::deserialize(buf).unwrap()
    )
}

impl Default for Server {
    fn default() -> Self {
        Self::new("127.0.0.1:1080", &[Methods::NoAuthenticationRequired])
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn server_test() {
        let server = Server::new("127.0.0.1:8000", &[Methods::UsernamePassword]).unwrap();
        let addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(3)).await;

        let mut connection = TcpStream::connect(&addr).await.unwrap();
        let estbl_req = EstablishRequest::new(&[Methods::NoAuthenticationRequired, Methods::UsernamePassword]);
        send(&mut connection, estbl_req).await.unwrap();
        let data = recv::<EstablishResponse>(&mut connection, &mut Vec::new()).await.unwrap();

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }
    
    #[tokio::test]
    async fn default_server_test() {
        let server = Server::default();
        let addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(3)).await;

        let mut connection = TcpStream::connect(&addr).await.unwrap();
        let estbl_req = EstablishRequest::new(&[Methods::NoAuthenticationRequired, Methods::UsernamePassword]);
        send(&mut connection, estbl_req).await.unwrap();
        let data = recv::<EstablishResponse>(&mut connection, &mut Vec::new()).await.unwrap();

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }
}