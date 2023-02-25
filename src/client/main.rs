use tokio::io::AsyncReadExt;

const CHAT_SERVER: &str = "localhost:7878";

#[tokio::main]
async fn main() {
    //connect to the server with tokio
    let mut stream = tokio::net::TcpStream::connect(CHAT_SERVER).await.unwrap();

    //create a channel to send messages to the server
    let (mut tx, mut rx) = tokio::io::split(stream);

    //create a channel to receive messages from the server
    let (mut tx2, mut rx2) = tokio::sync::mpsc::channel(32);

    //spawn a task to read messages from the server
    tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 1024];
            let n = rx.read(&mut buf).await.unwrap();
            if n == 0 {
                break;
            }
            let msg = String::from_utf8_lossy(&buf[..n]);
            println!("server => {}", msg);
        }
    });
}
