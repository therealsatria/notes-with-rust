use rusqlite::{Connection, params};
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use chrono::Utc;
use crate::functions::utils::encrypt_data;

pub fn add_note(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan catatan (max 255 char): ");
    let mut note = String::new();
    std::io::stdin().read_line(&mut note)?;
    let note = note.trim();

    println!("Masukkan prioritas (1: Tinggi, 2: Sedang, 3: Rendah, kosongkan untuk default Sedang): ");
    let mut prio_choice = String::new();
    std::io::stdin().read_line(&mut prio_choice)?;
    let prio_choice = prio_choice.trim();
    let prio_choice: i32 = if prio_choice.is_empty() {
        2 // Default ke Sedang jika input kosong
    } else {
        prio_choice.parse().unwrap_or(2) // Default ke Sedang jika parsing gagal
    };

    let priority = match prio_choice {
        1 => "Tinggi",
        2 => "Sedang",
        3 => "Rendah",
        _ => {
            println!("Pilihan tidak valid, menggunakan 'Sedang' sebagai default.");
            "Sedang"
        }
    };

    let encrypted_note = encrypt_data(note, key)?;
    let encrypted_priority = encrypt_data(priority, key)?;
    let timestamp = Utc::now().to_rfc3339();
    let created_at = timestamp.clone();
    let modified_at = timestamp;

    conn.execute(
        "INSERT INTO notes (note, priority, createdAt, modifiedAt) VALUES (?1, ?2, ?3, ?4)",
        params![encrypted_note, encrypted_priority, created_at, modified_at],
    ).context("Failed to insert note")?;
    println!("Catatan berhasil ditambahkan!");
    Ok(())
}