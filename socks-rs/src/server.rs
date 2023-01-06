//! # Server
//! SOCKS Server releated module
//! TODO: Add a better description

use std::net::{ToSocketAddrs, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::io;
use crate::{
    establish::{Method, EstablishRequest, EstablishResponse},
    request::{
        Request,
        Command,
        AddrType
    },
    reply::{
        Reply,
        ReplyOpt,
    },
    utils::*,
    SOCKS_VERSION,
};

/// The `Server` struct that holds the
/// SOCKS Server configuration
#[derive(Debug, Clone)]
pub struct Server {
    version: u8,
    supported_methods: Arc<Vec<Method>>,
    addr: SocketAddr,
}

impl Server {
    /// Constructs a new Server
    pub fn new<S>(addr: S, supported_methods: &[Method]) -> io::Result<Self>
    where
        S: ToSocketAddrs
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        Ok(
            Self {
                version: SOCKS_VERSION,
                supported_methods: Arc::new(supported_methods.to_vec()),
                addr
            }
        )
    }

    /// Start the server and listen for new connections
    pub async fn start(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        let supported_methods = self.supported_methods.clone();

        loop {
            let (stream, _) = listener.accept().await?;
            let delivery = Arc::new(Delivery::new(stream));
            let methods = supported_methods.clone();

            if Self::handle_establish(delivery.clone(), methods).await.is_err() {
                break Ok(());
            }

            tokio::spawn(async move {
                loop {
                    Self::handle_requests(delivery.clone()).await.unwrap();
                }
            });
        }
    }

    // TODO: implement the user and password authentication if selected
    async fn handle_establish(delivery: Arc<Delivery>, methods: Arc<Vec<Method>>) -> io::Result<()> {
        let estbl_req = delivery.recv::<EstablishRequest>(&mut Vec::with_capacity(100)).await?;
                
        if methods.iter().any(
            |x| estbl_req.methods().contains(x)
        ) {
            if let Some(&val) = methods.iter().max_by_key(|&&x| x as u8) {
                delivery.send(EstablishResponse::new(val)).await?;
            }
        } else {
            delivery.send(EstablishResponse::new(Method::NoAcceptableMethods)).await?;
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "NO ACCEPTABLE METHODS"))
        }

        Ok(())
    }

    async fn handle_requests(delivery: Arc<Delivery>) -> io::Result<()> {
        let request = delivery.recv::<Request>(&mut Vec::with_capacity(100)).await?;

        let socket_addr = delivery.address().await?;
        let ip = socket_addr.ip();
        let port = socket_addr.port();

        if *request.addr_type() == AddrType::DomainName {
            eprintln!("DomainName is not current supported");
        }

        match request.command() {
            Command::Connect => {
                let reply = Reply::new(ReplyOpt::Succeeded, *request.addr_type(), ip, port);
                delivery.send(reply).await?;

                let (dst_ip, dst_port) = request.socket_addr();
            
                let mut dest_stream = TcpStream::connect((dst_ip, dst_port)).await.unwrap();
                dest_stream.write_all("batata\n".as_bytes()).await.unwrap();

                // TODO: DATA PIPING
            }
            Command::Bind => panic!("No BIND command!"),
            Command::UdpAssociate => panic!("No UDP command!")
        };

        Ok(())
    }

}

impl Default for Server {
    fn default() -> Self {
        Self::new("127.0.0.1:1080", &[Method::NoAuthenticationRequired])
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn server_establish_test() {
        let server = Server::new("127.0.0.1:1080", &[Method::NoAuthenticationRequired]).unwrap();
        let addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(3)).await;

        let delivery = Delivery::new(TcpStream::connect(&addr).await.unwrap());
        let estbl_req = EstablishRequest::new(&[Method::NoAuthenticationRequired, Method::UsernamePassword]);
        delivery.send(estbl_req).await.unwrap();
        let data = delivery.recv::<EstablishResponse>(&mut Vec::new()).await.unwrap();

        assert_ne!(*data.method(), Method::NoAcceptableMethods);

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }

    #[tokio::test]
    async fn server_request_test() {
        let server = Server::new("127.0.0.1:1081", &[Method::NoAuthenticationRequired]).unwrap();
        let addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(3)).await;

        let delivery = Delivery::new(TcpStream::connect(&addr).await.unwrap());
        let estbl_req = EstablishRequest::new(&[Method::NoAuthenticationRequired, Method::UsernamePassword]);
        delivery.send(estbl_req).await.unwrap();
        let data = delivery.recv::<EstablishResponse>(&mut Vec::new()).await.unwrap();

        assert_ne!(*data.method(), Method::NoAcceptableMethods);

        let request = Request::new(Command::Connect, AddrType::IpV4, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        delivery.send::<Request>(request).await.unwrap();
        let data = delivery.recv::<Reply>(&mut Vec::with_capacity(100)).await.unwrap();
        
        println!("{data:?}");

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }
}