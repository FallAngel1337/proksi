use proksi::Server;
use std::{fs::File, io::Read};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("./examples/server_config.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let server: Server = serde_json::from_str(&contents)?;

    println!("Listening at {}", server.addr);
    server.start().await?;

    Ok(())
}
