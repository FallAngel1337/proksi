use socks_rs::{
    establish::{method, EstablishRequest, EstablishResponse},
    reply::{reply_opt, Reply},
    request::{addr_type, command, Request},
    Sendible, SOCKS_VERSION,
};
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use std::env;

fn help() -> ! {
    println!("proksi 0.1.0 - A rust proxy server");
    println!("Usage: ./proksi <addr>:<port> <auth>\n");
    println!("Arguments:");
    println!("<addr:port>\tThe address and port the proxy will bind");
    println!("<auth>     \tSupported authentication methods string (\"noauth,gssapi,userpasswd\")");
    std::process::exit(0);
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Server<'a> {
    version: u8,
    auth: &'a [u8],
    addr: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);

    let addr = args
        .next()
        .unwrap_or_else(|| help());

    let auth = args
        .next()
        .unwrap_or_else(|| help())
        .split(',')
        .map(|auth| match auth.to_ascii_lowercase().trim() {
            "noauth" => method::NO_AUTHENTICATION_REQUIRED,
            "gssapi" => method::GSSAPI,
            "userpasswd" => method::USERNAME_PASSWORD,
            _ => panic!("Invalid method")
        }).collect::<Vec<_>>();


    println!("Listening at {addr} ...");
    Server::new(&addr, &auth)?.start().await?;

    Ok(())
}

impl<'a> Server<'a> {
    /// Constructs a new Server
    pub fn new<S>(addr: S, auth: &'a [u8]) -> io::Result<Self>
    where
        S: ToSocketAddrs,
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        Ok(Self {
            version: SOCKS_VERSION,
            auth,
            addr,
        })
    }

    /// Start the server and listen for new connections
    pub async fn start(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;

        loop {
            let (mut stream, _) = listener.accept().await?;

            Self::handle_establish(&mut stream, self.auth).await?;
            Self::handle_requests(&mut stream).await?;
        }
    }

    // TODO: implement the user and password authentication if selected
    async fn handle_establish(stream: &mut TcpStream, methods: &[u8]) -> io::Result<()> {
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let establish_request = EstablishRequest::deserialize(&buf).unwrap();

        if methods
            .iter()
            .any(|x| establish_request.methods.contains(x))
        {
            if let Some(&method) = methods.iter().max_by_key(|&&x| x) {
                stream
                    .write_all(&EstablishResponse::new(method).serialize()?)
                    .await
                    .unwrap();
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
    use tokio::{
        time::{self, Duration},
        task::JoinHandle
    };

    #[tokio::test]
    async fn main_test() {
        let (addr, handler) = server_run("127.0.0.1:1080").await;
        let mut stream = TcpStream::connect(addr).await.unwrap();

        assert!(server_establish_test(&mut stream).await.is_ok());
        assert!(server_request_test(&mut stream).await.is_ok());

        handler.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(handler.is_finished());
    }

    async fn server_run(addr: impl ToSocketAddrs) -> (SocketAddr, JoinHandle<()>) {
        let server = Server::new(addr, &[method::NO_AUTHENTICATION_REQUIRED]).unwrap();
        let addr = server.addr;
        let handler = tokio::spawn(async move {
                server.start().await.unwrap()
            }
        );
        time::sleep(Duration::from_secs(2)).await;
        (addr, handler)
    }

    async fn server_establish_test(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let establish_request = EstablishRequest::new(&[
            method::NO_AUTHENTICATION_REQUIRED,
            method::USERNAME_PASSWORD,
        ]);

        stream.write_all(&establish_request.serialize()?).await?;

        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let establish_response = EstablishResponse::deserialize(&buf)?;

        assert_ne!(establish_response.method, method::NO_ACCEPTABLE_METHODS);

        println!("{establish_request:?}\n{establish_response:?}");

        Ok(())
    }

    async fn server_request_test(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        let listener_handeler = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = Vec::with_capacity(15);
            assert!(socket.read_buf(&mut buf).await.unwrap() > 0);
            assert_eq!(&buf, "secret_key".as_bytes());
            socket.write_all("secret_reponse".as_bytes()).await.unwrap();
            true
        });

        let request = Request::new(command::CONNECT, addr_type::IP_V4, &[127, 0, 0, 1], 8080);
        stream.write_all(&request.serialize()?).await?;

        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let reply = Reply::deserialize(&buf)?;
        
        assert_eq!(reply.rep, reply_opt::SUCCEEDED);
        println!("{request:?}\n{reply:?}");

        stream.write_all("secret_key".as_bytes()).await?;
        let mut secret_reponse = Vec::with_capacity(20);
        stream.read_buf(&mut secret_reponse).await?;
        println!("secret_response = {}", std::str::from_utf8(&secret_reponse)?);

        assert!(listener_handeler.await?);
        Ok(())
    }

    #[tokio::test]
    async fn server_curl_test() {
        use tokio::process::Command;
        let (_, handler) = server_run("127.0.0.1:1081").await;
        
        let cmd = Command::new("/bin/curl")
            .args(["--socks5", "localhost:1081", "google.com"])
            .output()
            .await
            .unwrap();
        
        assert!(cmd.status.success());

        handler.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(handler.is_finished());
    }
}
