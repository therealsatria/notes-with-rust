use rusqlite::Connection;
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use csv::WriterBuilder;
use std::fs::File;
use chrono::{DateTime, Utc};
use crate::functions::utils::{Note, decrypt_data};

pub fn export_to_csv(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    let mut stmt = conn.prepare("SELECT id, note, priority, createdAt, modifiedAt FROM notes")
        .context("Failed to prepare statement")?;
    let note_iter = stmt.query_and_then(
        [],
        |row| -> anyhow::Result<Note> {
            let encrypted_note: Vec<u8> = row.get(1)?;
            let encrypted_priority: Vec<u8> = row.get(2)?;
            let note = decrypt_data(&encrypted_note, key)?;
            let priority = decrypt_data(&encrypted_priority, key)?;
            let created_at_str: String = row.get(3)?;
            let modified_at_str: Option<String> = row.get(4)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))?;
            let modified_at = modified_at_str.map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
            }).transpose()?;
            Ok(Note {
                id: row.get(0)?,
                note,
                priority,
                created_at,
                modified_at,
            })
        }
    ).context("Failed to query notes for export")?;

    let file = File::create("notes_export.csv").context("Failed to create CSV file")?;
    let mut wtr = WriterBuilder::new().from_writer(file);

    wtr.write_record(&["id", "note", "priority", "createdAt", "modifiedAt"])
        .context("Failed to write CSV header")?;

    for note in note_iter {
        let note = note?;
        wtr.write_record(&[
            note.id.to_string(),
            note.note,
            note.priority,
            note.created_at.to_rfc3339(),
            note.modified_at.map_or(String::new(), |dt| dt.to_rfc3339()),
        ]).context("Failed to write CSV record")?;
    }

    wtr.flush().context("Failed to flush CSV writer")?;
    println!("Data berhasil diekspor ke 'notes_export.csv'!");
    Ok(())
}