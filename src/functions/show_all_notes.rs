use rusqlite::{Connection, params};
use textwrap::wrap;
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use chrono::{DateTime, Utc};
use crate::functions::utils::{Note, decrypt_data};

pub fn show_all_notes(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    let query = "SELECT id, note, priority, createdAt, modifiedAt FROM notes ORDER BY id ASC";

    let mut stmt = conn.prepare(query).context("Failed to prepare statement")?;

    let note_iter = stmt.query_and_then(
        params![],
        |row| -> anyhow::Result<Note> {
            let encrypted_note: Vec<u8> = row.get(1)
                .context("Failed to get note from row")?;
            let encrypted_priority: Vec<u8> = row.get(2)
                .context("Failed to get priority from row")?;
            let note = decrypt_data(&encrypted_note, key)
                .context("Failed to decrypt note")?;
            let priority = decrypt_data(&encrypted_priority, key)
                .context("Failed to decrypt priority")?;
            let created_at_str: String = row.get(3)
                .context("Failed to get createdAt from row")?;
            let modified_at_str: Option<String> = row.get(4)
                .context("Failed to get modifiedAt from row")?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .context("Failed to parse createdAt")?;
            let modified_at = modified_at_str.map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .context("Failed to parse modifiedAt")
            }).transpose()?;
            Ok(Note {
                id: row.get(0).context("Failed to get id from row")?,
                note,
                priority,
                created_at,
                modified_at,
            })
        }
    ).context("Failed to query all notes")?;

    println!("\nDaftar Semua Catatan (Urut berdasarkan ID):");
    println!("| {:<4} | {:<60} | {:<10} | {:<19} | {:<19} |", 
             "ID", "Note", "Priority", "Created At", "Modified At");
    println!("|------|--------------------------------------------------------------|------------|---------------------|---------------------|");

    for note in note_iter {
        let note = note?;
        let wrapped_note = wrap(&note.note, 60);
        for (i, line) in wrapped_note.iter().enumerate() {
            if i == 0 {
                println!(
                    "| {:<4} | {:<60} | {:<10} | {:<19} | {:<19} |",
                    note.id,
                    line,
                    note.priority,
                    note.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    note.modified_at.map_or("".to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                );
            } else {
                println!(
                    "| {:<4} | {:<60} | {:<10} | {:<19} | {:<19} |",
                    "", line, "", "", ""
                );
            }
        }
        println!("|------|--------------------------------------------------------------|------------|---------------------|---------------------|");
    }
    Ok(())
}