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

    println!("Masukkan prioritas (1: Tinggi, 2: Sedang, 3: Rendah): ");
    let mut prio_choice = String::new();
    std::io::stdin().read_line(&mut prio_choice)?;
    let prio_choice: i32 = prio_choice.trim().parse().unwrap_or(2);

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
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO notes (note, priority, createdAt) VALUES (?1, ?2, ?3)",
        params![encrypted_note, encrypted_priority, created_at],
    ).context("Failed to insert note")?;
    println!("Catatan berhasil ditambahkan!");
    Ok(())
}