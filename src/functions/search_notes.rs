use rusqlite::Connection;
use textwrap::wrap;
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use crate::functions::utils::{Note, decrypt_data};

pub fn search_notes(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan kata kunci untuk mencari catatan: ");
    let mut keyword = String::new();
    std::io::stdin().read_line(&mut keyword)?;
    let keyword = keyword.trim();

    let mut stmt = conn.prepare("SELECT id, note, priority FROM notes")
        .context("Failed to prepare statement")?;

    let note_iter = stmt.query_and_then(
        [],
        |row| -> anyhow::Result<Note> {
            let encrypted_note: Vec<u8> = row.get(1)
                .context("Failed to get note from row")?;
            let encrypted_priority: Vec<u8> = row.get(2)
                .context("Failed to get priority from row")?;
            let note = decrypt_data(&encrypted_note, key)
                .context("Failed to decrypt note")?;
            let priority = decrypt_data(&encrypted_priority, key)
                .context("Failed to decrypt priority")?;
            Ok(Note {
                id: row.get(0).context("Failed to get id from row")?,
                note,
                priority,
                created_at: chrono::Utc::now(), // Dummy
                modified_at: None, // Dummy
            })
        }
    ).context("Failed to query notes")?;

    let mut found = false;
    println!("\nHasil Pencarian untuk '{}':", keyword);
    println!("| {:<4} | {:<60} | {:<10} |", "ID", "Note", "Priority");
    println!("|------|--------------------------------------------------------------|------------|");

    for note in note_iter {
        let note = note?;
        if note.note.to_lowercase().contains(&keyword.to_lowercase()) {
            found = true;
            let wrapped_note = wrap(&note.note, 60);
            for (i, line) in wrapped_note.iter().enumerate() {
                if i == 0 {
                    println!("| {:<4} | {:<60} | {:<10} |", note.id, line, note.priority);
                } else {
                    println!("| {:<4} | {:<60} | {:<10} |", "", line, "");
                }
            }
            println!("|------|--------------------------------------------------------------|------------|");
        }
    }

    if !found {
        println!("Tidak ada catatan yang cocok dengan kata kunci '{}'.", keyword);
    }
    Ok(())
}