use std::sync::Arc;

use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use rusqlite::{params, Connection, Result as SqlResult};
use tokio::sync::Mutex;

pub fn init_user_database() -> SqlResult<Connection> {
    let conn = Connection::open("db.sqlite3")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            message TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;

    tracing::info!("user database initialized");
    Ok(conn)
}

pub async fn register_user(
    conn: &Arc<Mutex<Connection>>,
    username: &str,
    password: &str,
) -> SqlResult<()> {
    let hashed_password = hash(password, DEFAULT_COST).unwrap();
    conn.lock().await.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        [username, &hashed_password],
    )?;
    tracing::info!("registered user {}", username);
    Ok(())
}

pub async fn authenticate_user(
    conn: &Arc<Mutex<Connection>>,
    username: &str,
    password: &str,
) -> SqlResult<bool> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT password FROM users WHERE username = ?1")?;
    let stored_password: String = match stmt.query_row(params![username], |row| row.get(0)) {
        Ok(password) => password,
        Err(_) => return Ok(false),
    };

    let is_valid = verify(password, &stored_password).unwrap();
    tracing::info!("user {} authenticated: {}", username, is_valid);

    Ok(is_valid)
}

pub async fn store_message(
    conn: &Arc<Mutex<Connection>>,
    username: &str,
    message: &str,
) -> SqlResult<()> {
    let timestamp = Utc::now().to_string();
    conn.lock().await.execute(
        "INSERT INTO messages (username, message, timestamp) VALUES (?1, ?2, ?3)",
        params![username, message, timestamp],
    )?;
    Ok(())
}

pub async fn get_all_messages(conn: &Mutex<Connection>) -> Result<Vec<String>, rusqlite::Error> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT username, message FROM messages")?;
    let rows = stmt.query_map([], |row| {
        let username: String = row.get(0)?;
        let message: String = row.get(1)?;
        Ok(format!("{}: {}", username, message))
    })?;
    let mut messages = Vec::new();
    for message in rows {
        messages.push(message?);
    }
    Ok(messages)
}

pub async fn get_messages_by_user(
    conn: &Mutex<Connection>,
    username: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT message FROM messages WHERE username = ?1")?;
    let rows = stmt.query_map([username], |row| {
        let message: String = row.get(0)?;
        Ok(format!("{}: {}", username, message))
    })?;
    let mut messages = Vec::new();
    for message in rows {
        messages.push(message?);
    }
    Ok(messages)
}
