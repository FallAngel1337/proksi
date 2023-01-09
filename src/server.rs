//! # Server
//! SOCKS Server releated module
//! TODO: Add a better description

use socks_rs::{
    establish::{method, EstablishRequest, EstablishResponse},
    reply::{reply_opt, Reply},
    request::{addr_type, command, Request},
    Sendible,
    SOCKS_VERSION,
};
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

/// The `Server` struct that holds the
/// SOCKS Server configuration
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Server<'a> {
    version: u8,
    supported_methods: &'a [u8],
    addr: SocketAddr,
}

impl<'a> Server<'a> {
    /// Constructs a new Server
    pub fn new<S>(addr: S, supported_methods: &'a [u8]) -> io::Result<Self>
    where
        S: ToSocketAddrs,
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        Ok(Self {
            version: SOCKS_VERSION,
            supported_methods,
            addr,
        })
    }

    /// Start the server and listen for new connections
    pub async fn start(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;

        loop {
            let (mut stream, _) = listener.accept().await?;

            if Self::handle_establish(&mut stream, self.supported_methods)
                .await
                .is_err()
            {
                break Ok(());
            }

            Self::handle_requests(&mut stream).await.unwrap();
        }
    }

    // TODO: implement the user and password authentication if selected
    async fn handle_establish(
        stream: &mut TcpStream,
        methods: &[u8],
    ) -> io::Result<()> {
        
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let establish_request = EstablishRequest::deserialize(&buf).unwrap();

        if methods.iter().any(|x| establish_request.methods.contains(x)) {
            if let Some(&val) = methods.iter().max_by_key(|&&x| x) {
                stream.write_all(&establish_request.serialize()?).await.unwrap();
            }
        } else {
            stream
                .write_all(&EstablishResponse::new(method::NO_ACCEPTABLE_METHODS).serialize()?)
                .await?;
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "NO ACCEPTABLE METHODS",
            ));
        }

        Ok(())
    }

    async fn handle_requests(stream: &mut TcpStream) -> io::Result<()> {
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let request = Request::deserialize(&buf)?;

        let socket_addr = stream.peer_addr()?;
        let ip = match socket_addr.ip() {
            std::net::IpAddr::V4(ip) => ip.octets().to_vec(),
            std::net::IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let port = socket_addr.port();

        match request.cmd {
            command::CONNECT => {
                let reply = Reply::new(reply_opt::SUCCEEDED, request.atyp, &ip, port);
                stream.write_all(&reply.serialize()?).await?;

                let (dst_ip, dst_port) = (request.dst_addr, request.dst_port);

                let mut dst_stream = match request.atyp {
                    addr_type::IP_V4 => TcpStream::connect(SocketAddr::from((
                        TryInto::<[u8; 4]>::try_into(dst_ip).unwrap(),
                        dst_port,
                    )))
                    .await
                    .unwrap(),
                    addr_type::DOMAIN_NAME => panic!("No DOMAINNAME method yet"),
                    addr_type::IP_V6 => TcpStream::connect(SocketAddr::from((
                        TryInto::<[u8; 16]>::try_into(dst_ip).unwrap(),
                        dst_port,
                    )))
                    .await
                    .unwrap(),
                    _ => panic!("Invalid address type"),
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
            _ => panic!("Command {cmd} not available!", cmd = request.cmd),
        };
    }
}

impl Default for Server<'_> {
    fn default() -> Self {
        Self::new("127.0.0.1:1080", &[method::NO_AUTHENTICATION_REQUIRED]).unwrap()
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
        let estbl_req = EstablishRequest::new(&[
            method::NO_AUTHENTICATION_REQUIRED,
            method::USERNAME_PASSWORD,
        ]);
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
        let estbl_req = EstablishRequest::new(&[
            method::NO_AUTHENTICATION_REQUIRED,
            method::USERNAME_PASSWORD,
        ]);
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

        println!(
            "OUTPUT = {out:?} | {err}",
            out = cmd.stdout,
            err = std::str::from_utf8(&cmd.stderr).unwrap()
        );

        hdl.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(hdl.is_finished());
    }
}
