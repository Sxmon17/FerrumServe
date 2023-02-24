use std::io::prelude::*;
use std::net::TcpStream;

const ECHO_SERVER: &str = "localhost:7878";

#[tokio::main]
async fn main() {
    println!("Connecting to {}", ECHO_SERVER);
    if let Ok(stream) = TcpStream::connect(ECHO_SERVER) {
        println!(
            "connected to {}:{}",
            stream.local_addr().unwrap().ip(),
            stream.local_addr().unwrap().port()
        );
    } else {
        println!("Failed to connect to {}", ECHO_SERVER);
    }
}