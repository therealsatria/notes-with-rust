use rusqlite::Connection;
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Result;
use crate::functions::show_notes::show_notes;

pub fn refresh_data(conn: &Connection, key: &Key<Aes256Gcm>) -> Result<()> {
    show_notes(conn, key)?;
    println!("Data telah diperbarui.");
    Ok(())
}