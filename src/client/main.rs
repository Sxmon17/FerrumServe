use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use std::error::Error;

const CHAT_SERVER: &str = "127.0.0.1:7878";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await?;
    println!("Connected to the server: {}", CHAT_SERVER);

    loop {
        let (mut reader, mut writer) = stream.split();
        let input = String::new();
        let mut data = vec![0; 1024];

        tokio::select! {
            result = reader.read(&mut data) => {
                if result.unwrap() == 0 {
                    break;
                }
                println!("{}", String::from_utf8_lossy(&data));
            }
            result = writer.write(input.as_bytes()) => {
                result.unwrap();
            }
        }
    }
    Ok(())
}
