use serde::Deserialize;
use socks_rs::{
    auth::{AuthRequest, AuthResponse},
    establish::{method, EstablishRequest, EstablishResponse},
    reply::{reply_opt, Reply},
    request::{addr_type, command, Request},
    Sendible, SOCKS_VERSION,
};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub mod user;
use user::User;

#[macro_use]
mod macros {
    macro_rules! ip_octs {
        ($sock:expr) => {{
            match $sock.ip() {
                std::net::IpAddr::V4(ip) => (addr_type::IP_V4, ip.octets().to_vec()),
                std::net::IpAddr::V6(ip) => (addr_type::IP_V6, ip.octets().to_vec()),
            }
        }};
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

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    #[serde(skip)]
    version: u8,
    pub addr: SocketAddr,
    auth: Vec<u8>,
    #[serde(default)]
    allowed_users: Vec<User>,
}

impl Server {
    /// Constructs a new Server
    pub fn new<S>(addr: S, auth: Vec<u8>, allowed_users: Vec<User>) -> io::Result<Self>
    where
        S: ToSocketAddrs,
    {
        let addr = addr.to_socket_addrs()?.next().unwrap();
        Ok(Self {
            version: SOCKS_VERSION,
            auth,
            addr,
            allowed_users,
        })
    }

    /// Start the server and listen for new connections
    pub async fn start(self) -> io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        let server = Arc::new(self);

        loop {
            let (mut stream, addr) = listener.accept().await?;

            println!("Connection from {addr:?}");

            let server = Arc::clone(&server);
            tokio::spawn(async move {
                server
                    .establish_connection_handler(&mut stream)
                    .await
                    .unwrap_or_else(|err| {
                        eprintln!("Error: {err}");
                    })
            });
        }
    }

    async fn establish_connection_handler(
        self: Arc<Self>,
        stream: &mut TcpStream,
    ) -> io::Result<()> {
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let establish_request = EstablishRequest::deserialize(&buf).unwrap();

        let establish_method = self
            .auth
            .iter()
            .max_by_key(|&k| establish_request.methods.contains(k))
            .copied()
            .unwrap_or(method::NO_ACCEPTABLE_METHODS);

        stream
            .write_all(&EstablishResponse::new(establish_method).serialize()?)
            .await
            .unwrap();

        match establish_method {
            method::USERNAME_PASSWORD => self.auth_request(stream).await?,
            method::GSSAPI => panic!("No support for GSSAPI yet"),
            method::NO_ACCEPTABLE_METHODS => error!("NO ACCEPTABLE METHODS"),
            _ => (),
        };

        self.request_handler(stream).await
    }

    async fn auth_request(&self, stream: &mut TcpStream) -> io::Result<()> {
        use std::str;

        let mut buf = Vec::with_capacity(100);
        stream.read_buf(&mut buf).await?;

        let auth_request = AuthRequest::deserialize(&buf)?;

        let user = User::new(
            str::from_utf8(auth_request.uname).unwrap(),
            str::from_utf8(auth_request.passwd).unwrap(),
        );

        let response = AuthResponse::new(!self.allowed_users.contains(&user) as u8);

        stream.write_all(&response.serialize()?).await?;

        if response.status != 0 {
            error!(
                "({username}:{password}) WRONG user/password",
                username = user.username,
                password = user.password
            )
        }

        Ok(())
    }

    async fn request_handler(self: Arc<Self>, stream: &mut TcpStream) -> io::Result<()> {
        let mut buf = Vec::with_capacity(50);
        stream.read_buf(&mut buf).await?;
        let request = Request::deserialize(&buf)?;

        match request.cmd {
            command::CONNECT => self.connect_request(stream, request).await?,
            #[cfg(feature = "bind")]
            command::BIND => self.bind_request(stream).await?,
            #[cfg(not(feature = "bind"))]
            command::BIND => panic!("No BIND command!"),
            command::UDP_ASSOCIATE => panic!("No UDP command!"),
            cmd => error!("Command {cmd} not available!"),
        };

        Ok(())
    }

    async fn connect_request(
        &self,
        stream: &mut TcpStream,
        request: Request<'_>,
    ) -> io::Result<()> {
        let socket_addr = stream.local_addr()?;
        let (atyp, ip) = ip_octs!(socket_addr);
        let port = socket_addr.port();

        let (dst_ip, dst_port) = (request.dst_addr, request.dst_port);

        let mut reply = Reply::new(reply_opt::SUCCEEDED, atyp, &ip, port);

        let dst_socket = match request.atyp {
            addr_type::IP_V4 => {
                SocketAddr::from((TryInto::<[u8; 4]>::try_into(dst_ip).unwrap(), dst_port))
            }
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
            }
            #[cfg(not(feature = "dns-lookup"))]
            addr_type::DOMAIN_NAME => panic!("DOMAIN_NAME is not available"),
            addr_type::IP_V6 => {
                SocketAddr::from((TryInto::<[u8; 16]>::try_into(dst_ip).unwrap(), dst_port))
            }
            atyp => {
                reply.rep = reply_opt::ADDRESS_TYPE_NOT_SUPPORTED;
                stream.write_all(&reply.serialize()?).await?;
                error!("ADDRESS TYPE NOT SUPPORTED ({atyp})")
            }
        };

        stream.write_all(&reply.serialize()?).await?;

        let mut dst_stream = TcpStream::connect(dst_socket).await?;

        pipe(stream, &mut dst_stream).await;

        Ok(())
    }

    #[cfg(feature = "bind")]
    async fn bind_request(&self, stream: &mut TcpStream) -> io::Result<()> {
        use rand::Rng;

        let socket_addr = stream.local_addr()?;
        let ip = socket_addr.ip();
        let (atyp, bnd_addr) = ip_octs!(socket_addr);
        let bnd_port = {
            let mut rng = rand::thread_rng();
            rng.gen()
        };

        let bind_stream = TcpListener::bind((ip, bnd_port)).await?;

        let reply = Reply::new(reply_opt::SUCCEEDED, atyp, &bnd_addr, bnd_port);
        stream.write_all(&reply.serialize()?).await?;

        let (mut socket, addr) = bind_stream.accept().await?;
        println!("Got a BIND connection from {addr:?}");

        pipe(stream, &mut socket).await;

        Ok(())
    }
}

impl Default for Server {
    fn default() -> Self {
        Server::new("0.0.0.0:1080", vec![0], vec![]).unwrap()
    }
}

async fn pipe(src: &mut TcpStream, dst: &mut TcpStream) -> ! {
    let (mut src_read_half, mut src_write_half) = src.split();
    let (mut dst_read_half, mut dst_write_half) = dst.split();

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
