use proksi::Server;
use socks_rs::{
    establish::{method, EstablishRequest, EstablishResponse},
    reply::{reply_opt, Reply},
    request::{addr_type, command, Request},
    Sendible,
};
use std::net::SocketAddr;
use tokio::time::{self, Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[macro_use]
pub(crate) mod macros {
    macro_rules! start {
        ($s:expr) => {{
            tokio::spawn(async move { $s.start().await.unwrap() })
        }};
    }

    macro_rules! listener {
        ($addr:expr) => {{
            let listener = TcpListener::bind($addr).await.unwrap();
            tokio::spawn(async move {
                loop {
                    let (_, addr) = listener.accept().await.unwrap();
                    println!(">> {addr:?} <<");
                }
            })
        }};
    }
}

#[tokio::test]
async fn server_request() {
    let server_addr = "127.0.0.1:1080";

    let server = Server::default();
    let server_handler = start!(server);
    let listener_handler = listener!("127.0.0.1:8080");

    time::sleep(Duration::from_secs(1)).await;

    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    assert!(server_establish_test(&mut stream).await.is_ok());
    assert!(server_request_test(&mut stream).await.is_ok());

    #[cfg(feature = "bind")]
    {
        let mut bind_stream = TcpStream::connect(server_addr).await.unwrap();

        assert!(server_establish_test(&mut bind_stream).await.is_ok());
        assert!(server_bind_request_test(&mut bind_stream).await.is_ok());

        let mut buf = Vec::with_capacity(50);
        bind_stream.read_buf(&mut buf).await.unwrap();
        println!("buf = {buf:?}");
    }

    std::mem::drop(stream);

    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    assert!(server_establish_test(&mut stream).await.is_ok());
    assert!(server_request_domain_test(&mut stream).await.is_ok());

    server_handler.abort();
    listener_handler.abort();
    time::sleep(Duration::from_secs(1)).await;
    assert!(server_handler.is_finished());
    assert!(listener_handler.is_finished());
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
    let request = Request::new(command::CONNECT, addr_type::IP_V4, &[127, 0, 0, 1], 8080);
    stream.write_all(&request.serialize()?).await?;

    let mut buf = Vec::with_capacity(50);
    stream.read_buf(&mut buf).await?;
    let reply = Reply::deserialize(&buf)?;

    assert_eq!(reply.rep, reply_opt::SUCCEEDED);
    println!("{request:?}\n{reply:?}");

    Ok(())
}

async fn server_request_domain_test(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        command::CONNECT,
        addr_type::DOMAIN_NAME,
        &[10, 103, 111, 111, 103, 108, 101, 46, 99, 111, 109],
        80,
    );
    stream.write_all(&request.serialize()?).await?;

    let mut buf = Vec::with_capacity(50);
    stream.read_buf(&mut buf).await?;
    let reply = Reply::deserialize(&buf)?;

    assert_eq!(reply.rep, reply_opt::SUCCEEDED);
    println!("{request:?}\n{reply:?}");

    Ok(())
}

#[cfg(feature = "bind")]
async fn server_bind_request_test(
    stream: &mut TcpStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let bind_request = Request::new(command::BIND, addr_type::IP_V4, &[127, 0, 0, 1], 8080);
    stream.write_all(&bind_request.serialize()?).await?;

    let mut buf = Vec::with_capacity(50);
    stream.read_buf(&mut buf).await?;
    let reply = Reply::deserialize(&buf)?;

    assert_eq!(reply.rep, reply_opt::SUCCEEDED);
    println!("BIND >> {bind_request:?}\n{reply:?}");

    let mut conn = TcpStream::connect(SocketAddr::new(
        TryInto::<[u8; 4]>::try_into(reply.bnd_addr)?.into(),
        reply.bnd_port,
    ))
    .await?;

    conn.write_all(b"batatabanana").await?;

    Ok(())
}
