use crate::Shared;
use colored::Colorize;
use rusqlite::{Connection, Result as SqlResult};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

pub fn get_all_users(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT username FROM users")?;
    let rows = stmt.query_map([], |row| {
        let username: String = row.get(0)?;
        Ok(username)
    })?;

    let mut users = Vec::new();
    for row in rows {
        users.push(row?);
    }
    Ok(users)
}