use rusqlite::Connection;
use anyhow::Context;

pub fn init_db() -> anyhow::Result<Connection> {
    let conn = Connection::open("notes.db")
        .context("Failed to open database")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note BLOB NOT NULL,
            priority BLOB NOT NULL,
            createdAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            modifiedAt DATETIME
        )",
        [],
    ).context("Failed to create table")?;
    Ok(conn)
}