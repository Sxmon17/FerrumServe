use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::sync::broadcast;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

const CHAT_SERVER: &str = "127.0.0.1:7878";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(CHAT_SERVER).await.unwrap();
    println!("Listening on: {} ...", CHAT_SERVER);

    let (tx, _rx) = broadcast::channel(10);
    println!("Broadcast channel created");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New client: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            writer.write_all(format!("new message => {}", msg).as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
