use rusqlite::{Connection, params};
use anyhow::Context;

pub fn delete_note(conn: &Connection) -> anyhow::Result<()> {
    println!("Masukkan ID catatan yang akan dihapus: ");
    let mut id = String::new();
    std::io::stdin().read_line(&mut id)?;
    let id: i32 = id.trim().parse().unwrap_or(0);

    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .context("Failed to delete note")?;
    println!("Catatan dengan ID {} berhasil dihapus!", id);
    Ok(())
}