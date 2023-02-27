use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    println!("created stream");

    let result = stream.write_all(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());

    Ok(())
}
