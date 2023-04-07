mod commands;
mod database;
mod websocket;

use colored::*;
use futures::SinkExt;
use prettytable::{row, Table};
use rusqlite::Connection;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_span_events(FmtSpan::FULL)
        .init();

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    let listener = TcpListener::bind(&addr).await?;

    let ascii = r"
  _____
_/ ____\__________________ __ __  _____             ______ ______________  __ ____
\   __\/ __ \_  __ \_  __ \  |  \/     \   ______  /  ___// __ \_  __ \  \/ // __ \
 |  | \  ___/|  | \/|  | \/  |  /  Y Y  \ /_____/  \___ \\  ___/|  | \/\   /\  ___/
 |__|  \___  >__|   |__|  |____/|__|_|  /         /____  >\___  >__|    \_/  \___  >
           \/                         \/               \/     \/                 \/

    "
        .to_string()
        .bright_purple()
        .bold();
    println!("{}", ascii);

    tracing::info!("server running on {}", addr);

    let conn = Arc::new(Mutex::new(database::init_user_database()?));
    let state = Arc::new(Mutex::new(Shared::new()));

    tokio::spawn(websocket::websocket_proxy());

    loop {
        let (stream, addr) = listener.accept().await?;
        let state_clone = state.clone();
        let conn_clone = conn.clone();

        tokio::spawn(async move {
            tracing::debug!("accepted connection");

            if let Err(e) = process(state_clone, conn_clone, stream, addr).await {
                tracing::error!("an error occurred; error = {:?}", e);
            }
        });
    }
}

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

#[derive(Debug, Clone)]
struct Shared {
    peers: HashMap<SocketAddr, Tx>,
    usernames: HashMap<SocketAddr, String>,
}

struct Peer {
    lines: Framed<TcpStream, LinesCodec>,
    rx: Rx,
}

impl Shared {
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
            usernames: HashMap::new(),
        }
    }

    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }

    fn is_user_connected(&self, username: &str) -> bool {
        self.usernames.values().any(|u| u == username)
    }

    pub async fn get_addr_by_username(&self, username: &str) -> Option<String> {
        for (addr, _) in self.peers.iter() {
            if let Some(user) = self.usernames.get(addr) {
                if *user == username {
                    return Some(addr.to_string());
                }
            }
        }
        None
    }

}

impl Peer {
    async fn new(
        state: Arc<Mutex<Shared>>,
        lines: Framed<TcpStream, LinesCodec>,
        username: String,
    ) -> io::Result<Peer> {
        let addr = lines.get_ref().peer_addr()?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.insert(addr, tx);
        state.lock().await.usernames.insert(addr, username.clone());
        Ok(Peer { lines, rx })
    }
}

async fn process(
    state: Arc<Mutex<Shared>>,
    conn: Arc<Mutex<Connection>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut lines = Framed::new(stream, LinesCodec::new());

    lines
        .send(
            "Please enter 'register' or 'login': \n\rregister username password:"
                .blue()
                .to_string(),
        )
        .await?;

    let login = match lines.next().await {
        Some(Ok(line)) => line,
        _ => {
            tracing::error!(
            "Failed to get login information from {}. Client disconnected.",
            addr
        );
            return Ok(());
        }
    };
    let login_parts = login.split(' ').collect::<Vec<&str>>();

    if login_parts.len() < 3 {
        lines
            .send("Invalid input format. Please enter 'register' or 'login': \n\rregister username password:".red().to_string())
            .await?;
        return Ok(());
    }

    let register_or_login = login_parts[0];
    let username = login_parts[1];
    let password = login_parts[2];

    if register_or_login == "register" {
        match database::register_user(&conn, username, password).await {
            Ok(_) => {
                tracing::info!("registered user {}", username);
                lines
                    .send(format!(
                        "Registration successful, welcome {}!",
                        username.green().bold()
                    ))
                    .await?;
            }
            Err(e) => {
                lines.send(format!("Registration failed: {:?}", e)).await?;
                return Ok(());
            }
        }
    } else if register_or_login == "login" {
        let authenticated = database::authenticate_user(&conn, username, password).await?;
        if !authenticated {
            lines
                .send("Authentication failed, please try again.")
                .await?;
            return Ok(());
        }
    } else {
        lines
            .send("Invalid command, use 'register' or 'login' followed by username and password.")
            .await?;
        return Ok(());
    }

    lines
        .send("\n\rWelcome to the chat!".green().to_string())
        .await?;

    let mut peer = Peer::new(state.clone(), lines, username.to_string()).await?;

    {
        let mut state = state.lock().await;
        tracing::info!("{} joined the chat", username);
        state
            .broadcast(
                addr,
                format!("\n\r-> User joined: {0}\n\r", username.blue().bold(), ).as_str(),
            )
            .await;
    }

    loop {
        tokio::select! {
        Some(msg) = peer.rx.recv() => {
            peer.lines.send(&msg).await?;
        }
        result = peer.lines.next() => match result {
            Some(Ok(msg)) => {
                let mut state = state.lock().await;
                match msg.trim().split(' ').collect::<Vec<&str>>()[0] {
                    "/listusers" => {
                        let db_conn = conn.lock().await;
                        let users = commands::get_all_users(&db_conn).unwrap_or_default();

                        let mut table = Table::new();
                        table.add_row(row!["Username", "Status"]);

                        for user in users {
                            let is_connected = state.is_user_connected(&user);
                            let status = if is_connected { "Online".green() } else { "Offline".red() };
                            table.add_row(row![user, status]);
                        }

                        let mut response = Vec::new();
                        table.print(&mut response).unwrap();
                        let response = String::from_utf8(response).unwrap();

                        peer.lines.send(response).await?;
                        tracing::info!("{} requested a list of users", username);
                    }
                    "/whisper" => {
                        let mut parts = msg.splitn(3, ' ');
                        if let Some(_pm_cmd) = parts.next() {
                            if let Some(target_username) = parts.next() {
                                if let Some(private_message) = parts.next() {
                                    if let Some(target_addr_str) = state.get_addr_by_username(target_username).await {
                                        if let Ok(target_addr) = target_addr_str.parse::<SocketAddr>() {
                                            if let Some(target_tx) = state.peers.get(&target_addr) {
                                                let msg = format!("{}(whisper): {}", username.green().bold(), private_message);
                                                target_tx.send(msg).unwrap();
                                            }
                                        }
                                    } else {
                                        peer.lines.send("User not found or not connected.").await?;
                                    }
                                } else {
                                    peer.lines.send("Invalid private message format. Use /pm <username> <message>").await?;
                                }
                            } else {
                                peer.lines.send("Invalid private message format. Use /pm <username> <message>").await?;
                            }
                        }
                    }
                    _ => {
                        let msg = format!("{}: {}", username.green().bold(), msg);
                        state.broadcast(addr, &msg).await;
                    }
                }
            }
            Some(Err(e)) => {
                tracing::error!(
                    "an error occurred while processing messages for {}; error = {:?}",
                    username,
                    e
                );
            }
            None => break,
        },
    }
    }

    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);
        state.usernames.remove(&addr);

        tracing::info!("{} has left the chat", username);
        state
            .broadcast(
                addr,
                format!("\n\r<- User left: {0}\n\r", username.red().bold()).as_str(),
            )
            .await;
    }

    Ok(())
}