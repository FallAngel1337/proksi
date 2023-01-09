mod server;
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
    server::Server::new(&addr, &auth)?.start().await?;

    Ok(())
}