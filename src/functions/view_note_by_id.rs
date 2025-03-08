use rusqlite::{Connection, params};
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use chrono::{DateTime, Utc};
use crate::functions::utils::{Note, decrypt_data};
use crate::functions::edit_note::edit_note;
use crate::functions::delete_note::delete_note;

pub fn view_note_by_id(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan ID catatan yang ingin dilihat: ");
    let mut id = String::new();
    std::io::stdin().read_line(&mut id)?;
    let id: i32 = id.trim().parse().unwrap_or(0);

    let mut stmt = conn.prepare("SELECT id, note, priority, createdAt, modifiedAt FROM notes WHERE id = ?1")
        .context("Failed to prepare statement")?;
    let mut note_iter = stmt.query_and_then(
        params![id],
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
    ).context("Failed to query note by ID")?;

    if let Some(note) = note_iter.next() {
        let note = note?;
        println!("\nDetail Catatan:");
        println!("ID         : {}", note.id);
        println!("Catatan    : {}", note.note);
        println!("Prioritas  : {}", note.priority);
        println!("Dibuat     : {}", note.created_at.to_rfc3339());
        if let Some(modified_at) = note.modified_at {
            println!("Diperbarui : {}", modified_at.to_rfc3339());
        }
        println!("\nMenu:");
        println!("1. Edit Catatan");
        println!("2. Hapus Catatan");
        println!("3. Ganti Prioritas");
        println!("4. Kembali ke Menu Utama");
        println!("Pilih opsi (1-4): ");

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice)?;
        let choice: i32 = choice.trim().parse().unwrap_or(0);

        match choice {
            1 => edit_note(conn, key, Some(note.id))?,
            2 => delete_note(conn)?,
            3 => change_priority(conn, key, note.id)?,
            4 => println!("Kembali ke menu utama."),
            _ => println!("Pilihan tidak valid!"),
        }
    } else {
        println!("Catatan dengan ID {} tidak ditemukan!", id);
    }
    Ok(())
}

// Fungsi bantu untuk ganti prioritas (digunakan dalam view_note_by_id)
pub fn change_priority(conn: &Connection, key: &Key<Aes256Gcm>, id: i32) -> anyhow::Result<()> {
    println!("Masukkan prioritas baru (1: Tinggi, 2: Sedang, 3: Rendah): ");
    let mut prio_choice = String::new();
    std::io::stdin().read_line(&mut prio_choice)?;
    let prio_choice: i32 = prio_choice.trim().parse().unwrap_or(0);

    let priority = match prio_choice {
        1 => "Tinggi",
        2 => "Sedang",
        3 => "Rendah",
        _ => {
            println!("Pilihan tidak valid, prioritas tidak diubah.");
            return Ok(());
        }
    };

    let encrypted_priority = crate::functions::utils::encrypt_data(priority, key)?;
    let modified_at = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE notes SET priority = ?1, modifiedAt = ?2 WHERE id = ?3",
        params![encrypted_priority, modified_at, id],
    ).context("Failed to change priority")?;
    println!("Prioritas catatan dengan ID {} berhasil diperbarui!", id);
    Ok(())
}