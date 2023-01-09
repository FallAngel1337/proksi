mod server;

use socks_rs::establish::method;
use std::env;

use crate::server::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Aad a decent argument parser

    let mut args = env::args().skip(1);

    let addr = args
        .next()
        .unwrap_or("0.0.0.0:1080".to_string());

    let auth = args
        .next()
        .unwrap_or("noauth,gssapi,userpasswd".to_string())
        .split(',')
        .map(|auth| match auth.to_ascii_lowercase().trim() {
            "noauth" => method::NO_AUTHENTICATION_REQUIRED,
            "gssapi" => method::GSSAPI,
            "userpasswd" => method::USERNAME_PASSWORD,
            _ => panic!("Invalid method")
        }).collect::<Vec<_>>();


    println!("Listening at {addr} ...");
    server::Server::new(&addr, &auth, Some(&[User::new("fall", "pass")]))?.start().await?;

    Ok(())
}
