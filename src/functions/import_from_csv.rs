use rusqlite::{Connection, params};
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use csv::ReaderBuilder;
use std::fs::File;
use chrono::{DateTime, Utc};
use crate::functions::utils::encrypt_data;

pub fn import_from_csv(conn: &mut Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan path file CSV untuk diimpor (default: 'notes_import.csv'): ");
    let mut path = String::new();
    std::io::stdin().read_line(&mut path)?;
    let path = path.trim();
    let path = if path.is_empty() { "notes_import.csv" } else { path };

    let file = File::open(path).context("Failed to open CSV file")?;
    let mut rdr = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result.context("Failed to read CSV record")?;
        let id: i32 = record.get(0).unwrap_or("0").parse().unwrap_or(0);
        let note = record.get(1).unwrap_or("").to_string();
        let priority = record.get(2).unwrap_or("Sedang").to_string();
        let created_at = record.get(3)
            .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)))
            .unwrap_or_else(|| Ok(Utc::now()))
            .context("Failed to parse createdAt from CSV")?;
        let modified_at = record.get(4)
            .filter(|s| !s.is_empty())
            .map(|s| DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc)))
            .transpose()
            .context("Failed to parse modifiedAt from CSV")?;

        let encrypted_note = encrypt_data(&note, key)?;
        let encrypted_priority = encrypt_data(&priority, key)?;

        records.push((id, encrypted_note, encrypted_priority, created_at.to_rfc3339(), modified_at.map(|dt| dt.to_rfc3339())));
    }

    let tx = conn.transaction().context("Failed to start transaction")?;
    tx.execute("DELETE FROM notes", []).context("Failed to clear table before import")?;
    for (id, note, priority, created_at, modified_at) in records {
        if id == 0 {
            tx.execute(
                "INSERT INTO notes (note, priority, createdAt, modifiedAt) VALUES (?1, ?2, ?3, ?4)",
                params![note, priority, created_at, modified_at],
            ).context("Failed to insert note during import")?;
        } else {
            tx.execute(
                "INSERT INTO notes (id, note, priority, createdAt, modifiedAt) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, note, priority, created_at, modified_at],
            ).context("Failed to insert note with ID during import")?;
        }
    }
    tx.commit().context("Failed to commit transaction")?;

    println!("Data berhasil diimpor dari '{}'", path);
    Ok(())
}