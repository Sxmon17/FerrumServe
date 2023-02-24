use std::io::prelude::*;
use std::net::TcpStream;

const ECHO_SERVER: &str = "localhost:1685";

fn main() {
    if let Ok(mut stream) = TcpStream::connect(ECHO_SERVER) {
        println!("Connected to the server!");
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            stream.write(input.as_bytes()).unwrap();
            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read(&mut data) {
                Ok(_) => {
                    if let Ok(s) = std::str::from_utf8(&data) {
                        println!("Reply: {}", s);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        println!("Couldn't connect to server...");
    }
}