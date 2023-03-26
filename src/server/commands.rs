use rusqlite::{Connection, Result as SqlResult};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn list_users(conn: &Arc<Mutex<Connection>>) -> SqlResult<Vec<String>> {
    let query = "SELECT username FROM users";
    let conn_lock = conn.lock().await;
    let mut stmt = conn_lock.prepare(query)?;
    let rows = stmt.query_map([], |row| {
        let username: String = row.get(0)?;
        Ok(username)
    })?;

    let mut user_list = Vec::new();
    for user in rows {
        user_list.push(user?);
    }

    Ok(user_list)
}
