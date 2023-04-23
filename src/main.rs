use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use colored::*;
use futures::SinkExt;
use prettytable::{row, Table};
use rusqlite::{Connection};
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::message::Message;

mod commands;
mod database;
mod message;
mod websocket;

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
    color: Color,
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
        Ok(Peer {
            lines,
            rx,
            color: Color::Green,
        })
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
        if state.lock().await.is_user_connected(username) {
            lines
                .send("User already connected, please try again.")
                .await?;
            return Ok(());
        }
        if database::get_user_role(&conn, username).await? == "banned" {
            lines
                .send("You are banned, please try again later.")
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
                format!("\n\r-> User joined: {0}\n\r", username.blue().bold(),).as_str(),
            )
            .await;
    }

    loop {
        tokio::select! {
            Some(msg) = peer.rx.recv() => {
                peer.lines.send(&msg).await?;
                if msg == "You have been banned.".red().to_string() {
                    break;
                }
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
                                        peer.lines.send("Invalid private message format. Use /whisper <username> <message>").await?;
                                    }
                                } else {
                                    peer.lines.send("Invalid private message format. Use /whisper <username> <message>").await?;
                                }
                            }
                        }
                        "/history" => {
                            let mut args = msg.split_whitespace().skip(1);
                            match args.next() {
                                Some("all") => {
                                    let messages = database::get_all_messages(&conn).await.unwrap_or_default();
                                    peer.lines.send(commands::format_message_history("all users", &messages)).await?;
                                }
                                Some(username) => {
                                    let messages = database::get_messages_by_user(&conn, username).await.unwrap_or_default();
                                    peer.lines.send(commands::format_message_history(username, &messages)).await?;
                                }
                                None => {
                                    let messages = database::get_messages_by_user(&conn, username).await.unwrap_or_default();
                                    peer.lines.send(commands::format_message_history(username, &messages)).await?;
                                }
                            }
                        }
                        "/changepw" => {
                            let mut parts = msg.splitn(3, ' ');
                            if let Some(_cmd) = parts.next() {
                                if let Some(old_password) = parts.next() {
                                    if let Some(new_password) = parts.next() {
                                        if database::authenticate_user(&conn, username, old_password).await? {
                                            database::update_user_password(&conn, username, new_password).await?;
                                            peer.lines.send("Password updated successfully.".green().to_string()).await?;
                                        } else {
                                            peer.lines.send("Incorrect password. Please try again.".red().to_string()).await?;
                                        }
                                    } else {
                                        peer.lines.send("Invalid command format. Use /changepassword <old_password> <new_password>").await?;
                                    }
                                } else {
                                    peer.lines.send("Invalid command format. Use /changepassword <old_password> <new_password>").await?;
                                }
                            }
                        }
                        "/color" => {
                            let mut parts = msg.split_whitespace().skip(1);
                            if let Some(color_name) = parts.next() {
                                match Color::from_str(color_name) {
                                    Ok(color) => {
                                        peer.color = color;
                                        peer.lines.send(format!("Text color changed to {}.", color_name.green())).await?;
                                    }
                                    Err(_) => {
                                        peer.lines.send("Invalid color. Please provide a valid color name.".red().to_string()).await?;
                                    }
                                }
                            } else {
                                peer.lines.send("Invalid command format. Use /color <color_name>").await?;
                            }
                        }
                        "/admin" => {
                            let mut password = msg.split_whitespace().skip(1);

                            let admin_pw = env::args()
                                .nth(2)
                                .unwrap_or_else(|| "admin".to_string());

                            if let Some(password) = password.next() {
                                if password == admin_pw {
                                    database::change_role(&conn, username, "admin").await?;
                                    peer.lines.send("You are now an admin.".green().to_string()).await?;
                                } else {
                                    peer.lines.send("Incorrect password. Please try again.".red().to_string()).await?;
                                }
                            } else {
                                peer.lines.send("Invalid command format. Use /admin <password>").await?;
                            }
                        }
                        "/ban" => {
                            if database::get_user_role(&conn, username).await? == "admin" {
                                let mut parts = msg.split_whitespace().skip(1);
                                if let Some(username) = parts.next() {
                                    database::change_role(&conn, username, "banned").await?;
                                    if let Some(addr) = state.get_addr_by_username(username).await {
                                        if let Ok(addr) = addr.parse::<SocketAddr>() {
                                            if let Some(tx) = state.peers.get(&addr) {
                                                tx.send("You have been banned.".red().to_string()).unwrap();
                                            }
                                        }
                                    }
                                    peer.lines.send(format!("{} has been banned.", username.green())).await?;
                                } else {
                                    peer.lines.send("Invalid command format. Use /ban <username>").await?;
                                }
                            } else {
                                peer.lines.send("You are not an admin.".red().to_string()).await?;
                            }
                        }
                        "/unban" => {
                            if database::get_user_role(&conn, username).await? == "admin" {
                                let mut parts = msg.split_whitespace().skip(1);
                                if let Some(username) = parts.next() {
                                    database::change_role(&conn, username, "user").await?;
                                    peer.lines.send(format!("{} has been unbanned.", username.green())).await?;
                                } else {
                                    peer.lines.send("Invalid command format. Use /unban <username>").await?;
                                }
                            } else {
                                peer.lines.send("You are not an admin.".red().to_string()).await?;
                            }
                        }
                        "/mute" => {
                            if database::get_user_role(&conn, username).await? == "admin" {
                                let mut parts = msg.split_whitespace().skip(1);
                                if let Some(username) = parts.next() {
                                    database::change_role(&conn, username, "muted").await?;
                                    peer.lines.send(format!("{} has been muted.", username.green())).await?;
                                } else {
                                    peer.lines.send("Invalid command format. Use /mute <username>").await?;
                                }
                            } else {
                                peer.lines.send("You are not an admin.".red().to_string()).await?;
                            }
                        }
                        "/unmute" => {
                            if database::get_user_role(&conn, username).await? == "admin" {
                                let mut parts = msg.split_whitespace().skip(1);
                                if let Some(username) = parts.next() {
                                    database::change_role(&conn, username, "user").await?;
                                    peer.lines.send(format!("{} has been unmuted.", username.green())).await?;
                                } else {
                                    peer.lines.send("Invalid command format. Use /unmute <username>").await?;
                                }
                            } else {
                                peer.lines.send("You are not an admin.".red().to_string()).await?;
                            }
                        }
                        "/help" => {
                            let mut response = Vec::new();
                            let mut table = Table::new();
                            table.add_row(row!["Command", "Description"]);
                            table.add_row(row!["/listusers", "List all users"]);
                            table.add_row(row!["/history [blank, username, all]", "Show msg history"]);
                            table.add_row(row!["/whisper <username> <message>", "Send a private message to a user"]);
                            table.add_row(row!["/changepw <old_password> <new_password>", "Change your password"]);
                            table.add_row(row!["/color <color_name>", "Change your username color"]);
                            table.add_row(row!["/admin <password>", "Become an admin"]);
                            table.add_row(row!["/ban <username>", "Ban a user"]);
                            table.add_row(row!["/unban <username>", "Unban a user"]);
                            table.add_row(row!["/mute <username>", "Mute a user"]);
                            table.add_row(row!["/unmute <username>", "Unmute a user"]);
                            table.add_row(row!["/help", "Show this help message"]);
                            table.print(&mut response).unwrap();
                            let response = String::from_utf8(response).unwrap();
                            peer.lines.send(response).await?;
                        }
                        _ => {
                            if database::get_user_role(&conn, username).await? == "muted" {
                                peer.lines.send("You are muted.".red().to_string()).await?;
                            } else {
                                let msg = Message::from_input(username.to_string(), msg.to_string());
                                database::store_message(&conn, &msg).await.unwrap_or_else(|e| {
                                    tracing::error!("Failed to store message: {:?}", e);
                                });
                                let formatted_msg = msg.format(peer.color);
                                state.broadcast(addr, &formatted_msg).await;
                            }
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
