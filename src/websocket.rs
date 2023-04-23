use futures::SinkExt;
use futures::StreamExt;
use futures::TryStreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tungstenite::Message;

pub async fn websocket_proxy() {
    let ws_listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    tracing::info!("websocket proxy listening on 127.0.0.1:8081");

    loop {
        let (stream, _) = ws_listener.accept().await.unwrap();
        let ws_stream = accept_async(stream).await.unwrap();
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let tcp_stream = TcpStream::connect("127.0.0.1:6142").await.unwrap();
        let (mut tcp_reader, mut tcp_writer) = tcp_stream.into_split();

        // Forward messages from WebSocket to TCP
        tokio::spawn(async move {
            while let Some(msg) = ws_receiver.try_next().await.unwrap() {
                if let Message::Text(text) = msg {
                    tcp_writer.write_all(text.as_bytes()).await.unwrap();
                }
            }
        });

        // Forward messages from TCP to WebSocket
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                let n = tcp_reader.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }

                let text = String::from_utf8_lossy(&buf[..n]);
                ws_sender
                    .send(Message::Text(text.into_owned()))
                    .await
                    .unwrap();
            }
        });
    }
}
