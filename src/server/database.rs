use std::sync::Arc;
use tokio::sync::Mutex;
use rusqlite::{Connection, params, Result as SqlResult};
use bcrypt::{DEFAULT_COST, hash, verify};

pub fn init_user_database() -> SqlResult<Connection> {
    let conn = Connection::open("user_database.sqlite3")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL
        )",
        [],
    )?;
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
    Ok(())
}

pub async fn authenticate_user(conn: &Arc<Mutex<Connection>>, username: &str, password: &str) -> SqlResult<bool> {
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT password FROM users WHERE username = ?1")?;
    let stored_password: String = match stmt.query_row(params![username], |row| row.get(0)) {
        Ok(password) => password,
        Err(_) => return Ok(false),
    };

    let is_valid = verify(password, &stored_password).unwrap();
    Ok(is_valid)
}
