use socks_rs::{
    establish::{method, EstablishRequest, EstablishResponse},
    reply::{reply_opt, Reply},
    request::{addr_type, command, Request},
    auth::{AuthRequest, AuthResponse},
    Sendible, SOCKS_VERSION,
};
use std::net::{SocketAddr, ToSocketAddrs};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[macro_use]
mod macros {
    macro_rules! ip_octs {
        ($sock:expr) => {
            {
                match $sock.ip() {
                    std::net::IpAddr::V4(ip) => (addr_type::IP_V4, ip.octets().to_vec()),
                    std::net::IpAddr::V6(ip) => (addr_type::IP_V6, ip.octets().to_vec()),
                }
            }
        };
    }

    macro_rules! error {
        ($($msg:tt)*) => {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    format_args!($($msg)*).to_string()
            ))
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    username: String,
    password: String
}

// TODO: Parse from a config file
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Server<'a> {
    version: u8,
    auth: &'a [u8],
    addr: SocketAddr,
    allowed_users: Option<Vec<User>>
}

#[allow(missing_docs, unused)]
impl User {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string()
        }
    }
}

impl<'a> Server<'a> {
    /// Constructs a new Server
    pub fn new<S>(addr: S, auth: &'a [u8], allowed_users: Option<Vec<User>>) -> io::Result<Self>
    where
        S: ToSocketAddrs,
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        Ok(Self {
            version: SOCKS_VERSION,
            auth,
            addr,
            allowed_users
        })
    }

    /// Start the server and listen for new connections
    pub async fn start(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;

        loop {
            let (mut stream, _) = listener.accept().await?;

            self.establish_connection_handler(&mut stream).await?;
            self.request_handler(&mut stream).await?;
        }
    }
    
    async fn establish_connection_handler(&self, stream: &mut TcpStream) -> io::Result<()> {
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let establish_request = EstablishRequest::deserialize(&buf).unwrap();

        let establish_method = self.auth
            .iter()
            .max_by_key(|&k| establish_request.methods.contains(k))
            .copied()
            .unwrap_or(method::NO_ACCEPTABLE_METHODS);

        stream
            .write_all(&EstablishResponse::new(establish_method).serialize()?)
            .await
            .unwrap();

        match establish_method {
            method::USERNAME_PASSWORD => self.auth_request(
                stream, 
                self.allowed_users.as_ref().unwrap()
            ).await?,
            method::GSSAPI => panic!("No support for GSSAPI yet"),
            method::NO_ACCEPTABLE_METHODS => error!("NO ACCEPTABLE METHODS"),
            _ => ()
        };

        Ok(())
    }

    async fn auth_request(&self, stream: &mut TcpStream, users_list: &[User]) -> io::Result<()> {
        use std::str;

        let mut buf = Vec::with_capacity(100);
        stream.read_buf(&mut buf).await?;

        let auth_request = AuthRequest::deserialize(&buf)?;

        let user = User::new(str::from_utf8(auth_request.uname).unwrap(), str::from_utf8(auth_request.passwd).unwrap());

        let response = AuthResponse::new(!users_list.contains(&user) as u8);

        stream.write_all(&response.serialize()?).await?;

        if response.status != 0 {
            error!("USER NOT FOUND")
        }

        Ok(())
    }

    async fn request_handler(&self, stream: &mut TcpStream) -> io::Result<()> {
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let request = Request::deserialize(&buf)?;

        match request.cmd {
            command::CONNECT => self.connect_request(stream, request).await?,
            command::BIND => panic!("No BIND command!"),
            command::UDP_ASSOCIATE => panic!("No UDP command!"),
            cmd => error!("Command {cmd} not available!"),
        };

        Ok(())
    }

    async fn connect_request(&self, stream: &mut TcpStream, request: Request<'_>) -> io::Result<()>{
        let socket_addr = stream.local_addr()?;
        let (atyp, ip) = ip_octs!(socket_addr);
        let port = socket_addr.port();

        let (dst_ip, dst_port) = (request.dst_addr, request.dst_port);

        let mut reply = Reply::new(reply_opt::SUCCEEDED, atyp, &ip, port);

        let dst_socket = match request.atyp {
            addr_type::IP_V4 => SocketAddr::from((
                TryInto::<[u8; 4]>::try_into(dst_ip).unwrap(),
                dst_port
            )),
            #[cfg(feature = "dns-lookup")]
            addr_type::DOMAIN_NAME => {
                let host = std::str::from_utf8(dst_ip).unwrap().trim();
                let resolved_list = dns_lookup::lookup_host(host)?;
                let resolved = resolved_list.first().unwrap();

                format!("{resolved}:{dst_port}")
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap()
            },
            #[cfg(not(feature = "dns-lookup"))]
            addr_type::DOMAIN_NAME => panic!("DOMAIN_NAME is not available"),
            addr_type::IP_V6 => SocketAddr::from((
                TryInto::<[u8; 16]>::try_into(dst_ip).unwrap(),
                dst_port,
            )),
            atyp => {
                reply.rep = reply_opt::ADDRESS_TYPE_NOT_SUPPORTED;
                stream.write_all(&reply.serialize()?).await?;
                error!("ADDRESS TYPE NOT SUPPORTED ({atyp})")
            }
        };

        stream.write_all(&reply.serialize()?).await?;

        let mut dst_stream = TcpStream::connect(dst_socket).await?;

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
}

impl Default for Server<'_> {
    fn default() -> Self {
        Self::new("127.0.0.1:1080", &[method::NO_AUTHENTICATION_REQUIRED], None).unwrap()
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
    async fn server_request() {
        let (addr, handler) = default_server_run(None).await;
        let mut stream = TcpStream::connect(addr).await.unwrap();
        
        assert!(server_establish_test(&mut stream).await.is_ok());
        assert!(server_request_test(&mut stream).await.is_ok());
        
        handler.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(handler.is_finished());
    }

    #[tokio::test]
    async fn server_request_domain() -> Result<(), Box<dyn std::error::Error>> {
        let (addr, handler) = default_server_run(Some("127.0.0.1:1081")).await;
        let mut stream = TcpStream::connect(addr).await.unwrap();

        assert!(server_establish_test(&mut stream).await.is_ok());

        let request = Request::new(command::CONNECT, addr_type::DOMAIN_NAME,
             &[10, 103, 111, 111, 103, 108, 101, 46, 99, 111, 109], 80);
        stream.write_all(&request.serialize()?).await?;

        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let reply = Reply::deserialize(&buf)?;

        assert_eq!(reply.rep, reply_opt::SUCCEEDED);
        println!("{request:?}\n{reply:?}");

        handler.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(handler.is_finished());

        Ok(())
    }

    async fn default_server_run(addr: Option<&str>) -> (SocketAddr, JoinHandle<()>) {
        let server = Server::new(addr.unwrap_or("127.0.0.1:1080"), &[method::NO_AUTHENTICATION_REQUIRED], None).unwrap();
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
        let (_, handler) = default_server_run(Some("127.0.0.1:1082")).await;
        
        let cmd = Command::new("/bin/curl")
            .args(["--socks5", "localhost:1082", "google.com"])
            .output()
            .await
            .unwrap();
        
        assert!(cmd.status.success());

        handler.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(handler.is_finished());
    }

    #[tokio::test]
    async fn server_curl_userpass_test() {
        use tokio::process::Command;

        let (username, password ) = ("admin", "admin");
        let user = User::new(username, password);

        let server = Server::new("127.0.0.1:1083", &[method::USERNAME_PASSWORD], Some(vec![user])).unwrap();

        let handler = tokio::spawn(async move {
                server.start().await.unwrap()
            }
        );
        time::sleep(Duration::from_secs(2)).await;
        
        let cmd = Command::new("/bin/curl")
            .args(["--proxy", "socks5://admin:admin@localhost:1083", "google.com"])
            .output()
            .await
            .unwrap();
        
        assert!(cmd.status.success());

        handler.abort();
        time::sleep(Duration::from_secs(1)).await;
        assert!(handler.is_finished());
    }
}
