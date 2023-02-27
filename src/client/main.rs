use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    println!("created stream");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        stream.write_all(input.as_bytes()).await?;
    }
}
