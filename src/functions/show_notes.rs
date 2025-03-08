use rusqlite::{Connection, params};
use textwrap::wrap;
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use chrono::{DateTime, Utc};
use std::env;
use crate::functions::utils::{Note, decrypt_data};

pub fn show_notes(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    let limit = env::var("SHOW_LIMIT")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<i64>()
        .unwrap_or(10);

    let order_by = env::var("SHOW_ORDER_BY")
        .unwrap_or_else(|_| "createdAt".to_string())
        .to_lowercase();

    let order_column = match order_by.as_str() {
        "id" => "id DESC",
        "createdat" => "createdAt DESC",
        "modifiedat" => "modifiedAt DESC",
        _ => {
            println!("SHOW_ORDER_BY tidak valid di .env, menggunakan default 'createdAt'");
            "createdAt DESC"
        }
    };

    let query = format!(
        "SELECT id, note, priority, createdAt, modifiedAt FROM notes ORDER BY {} LIMIT ?1",
        order_column
    );

    let mut stmt = conn.prepare(&query).context("Failed to prepare statement")?;

    let note_iter = stmt.query_and_then(
        params![limit],
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
    ).context("Failed to query notes")?;

    println!("\nDaftar Catatan (Limit: {}, Order By: {}):", limit, order_by);
    println!("| {:<4} | {:<60} | {:<10} |", "ID", "Note", "Priority");
    println!("|------|--------------------------------------------------------------|------------|");

    for note in note_iter {
        let note = note?;
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
    Ok(())
}