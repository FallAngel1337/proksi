//! # Server
//! SOCKS Server releated module
//! TODO: Add a better description

use std::net::{ToSocketAddrs, SocketAddr};
use tokio::{net::{TcpListener, TcpStream}, io::AsyncReadExt};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::io;
use crate::{
    establish::{method, EstablishRequest, EstablishResponse},
    request::{
        Request,
        command,
        addr_type,
    },
    reply::{
        Reply,
        reply_opt,
    },
    utils::*,
    SOCKS_VERSION,
};

/// The `Server` struct that holds the
/// SOCKS Server configuration
#[derive(Debug, Clone)]
pub struct Server {
    version: u8,
    supported_methods: Arc<Vec<u8>>,
    addr: SocketAddr,
}

impl Server {
    /// Constructs a new Server
    pub fn new<S>(addr: S, supported_methods: &[u8]) -> io::Result<Self>
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
            let (mut stream, _) = listener.accept().await?;
            let mut delivery = Delivery::new(&mut stream);
            let methods = supported_methods.clone();

            if Self::handle_establish(&mut delivery, methods).await.is_err() {
                break Ok(());
            }

            Self::handle_requests(stream).await.unwrap();
        }
    }

    // TODO: implement the user and password authentication if selected
    async fn handle_establish(delivery: &mut Delivery<'_>, methods: Arc<Vec<u8>>) -> io::Result<()> {
        let estbl_req = delivery.recv::<EstablishRequest>().await.unwrap();
                
        if methods.iter().any(
            |x| estbl_req.methods().contains(x)
        ) {
            if let Some(&val) = methods.iter().max_by_key(|&&x| x) {
                delivery.send(EstablishResponse::new(val)).await?;
            }
        } else {
            delivery.send(EstablishResponse::new(method::NO_ACCEPTABLE_METHODS)).await?;
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "NO ACCEPTABLE METHODS"))
        }

        Ok(())
    }

    async fn handle_requests(mut stream: TcpStream) -> io::Result<()> {
        let mut delivery = Delivery::new(&mut stream);
        let request = delivery.recv::<Request>().await?;

        let socket_addr = delivery.get_ref().peer_addr()?;
        let ip = match socket_addr.ip() {
            std::net::IpAddr::V4(ip) => ip.octets().to_vec(),
            std::net::IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let port = socket_addr.port();

        match request.command() {
            command::CONNECT => {
                let reply = Reply::new(reply_opt::SUCCEEDED, request.addr_type(), &ip, port);
                delivery.send(reply).await?;

                let (dst_ip, dst_port) = request.socket_addr();

                let mut dst_stream = match request.addr_type() {
                    addr_type::IP_V4 => TcpStream::connect(SocketAddr::from((TryInto::<[u8; 4]>::try_into(dst_ip).unwrap(), dst_port))).await.unwrap(),
                    addr_type::DOMAIN_NAME => panic!("No DOMAINNAME method yet"),
                    addr_type::IP_V6 => TcpStream::connect(SocketAddr::from((TryInto::<[u8; 16]>::try_into(dst_ip).unwrap(), dst_port))).await.unwrap(),
                    _ => panic!("Invalid address type")
                };
            
                let (mut dst_read_half, mut dst_write_half) = dst_stream.split();
                let (mut src_read_half, mut src_write_half) = stream.split();

                loop {
                    tokio::select! {
                        _ = async {
                            let mut buf = Vec::with_capacity(512);
                            src_read_half.read_buf(&mut buf).await.unwrap();
                            dst_write_half.write_all(&buf).await.unwrap();
                        } => { }
                        
                        _ = async {
                            let mut buf = Vec::with_capacity(512);
                            dst_read_half.read_buf(&mut buf).await.unwrap();
                            src_write_half.write_all(&buf).await.unwrap();
                        } => { }
                    }
                }
            }
            command::BIND => panic!("No BIND command!"),
            command::UDP_ASSOCIATE => panic!("No UDP command!"),
            _ => panic!("Command {cmd} not available!", cmd = request.command()),
        };
    }

}

impl Default for Server {
    fn default() -> Self {
        Self::new("127.0.0.1:1080", &[method::NO_AUTHENTICATION_REQUIRED])
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn server_establish_test() {
        let server = Server::new("127.0.0.1:1080", &[method::NO_AUTHENTICATION_REQUIRED]).unwrap();
        let addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(2)).await;

        let mut stream = TcpStream::connect(&addr).await.unwrap();
        let estbl_req = EstablishRequest::new(&[method::NO_AUTHENTICATION_REQUIRED, method::USERNAME_PASSWORD]);
        let mut delivery = Delivery::new(&mut stream);
        delivery.send(estbl_req).await.unwrap();
        let data = delivery.recv::<EstablishResponse>().await.unwrap();

        assert_ne!(*data.method(), method::NO_ACCEPTABLE_METHODS);
        
        println!("{data:?}");

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }

    #[tokio::test]
    async fn server_request_test() {
        let server = Server::new("127.0.0.1:1081", &[method::NO_AUTHENTICATION_REQUIRED]).unwrap();
        let addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(2)).await;

        let mut stream = TcpStream::connect(&addr).await.unwrap();
        let estbl_req = EstablishRequest::new(&[method::NO_AUTHENTICATION_REQUIRED, method::USERNAME_PASSWORD]);
        let mut delivery = Delivery::new(&mut stream);
        delivery.send(estbl_req).await.unwrap();
        let data = delivery.recv::<EstablishResponse>().await.unwrap();

        assert_ne!(*data.method(), method::NO_ACCEPTABLE_METHODS);
        
        println!("{data:?}");

        let request = Request::new(command::CONNECT, addr_type::IP_V4, &[127, 0, 0, 1], 8080);
        delivery.send::<Request>(request).await.unwrap();
        let data = delivery.recv::<Reply>().await.unwrap();
        
        println!("{data:?}");

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }

    #[tokio::test]
    #[ignore = "not running curl"]
    async fn server_curl_test() {
        use std::process::Command;
        let server = Server::new("127.0.0.1:1082", &[method::NO_AUTHENTICATION_REQUIRED]).unwrap();
        let _addr = server.addr;

        let hdl = tokio::spawn(async move { server.start().await.unwrap() });
        
        time::sleep(Duration::from_secs(2)).await;

        let cmd = Command::new("/bin/curl")
            .args(&["-v", "--socks5", "localhost:1082", "google.com"])
            .output()
            .unwrap();

        println!("OUTPUT = {out:?} | {err}", out = cmd.stdout, err = std::str::from_utf8(&cmd.stderr).unwrap());

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }
}