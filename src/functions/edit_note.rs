use rusqlite::{Connection, params};
use aes_gcm::{Aes256Gcm, Key};
use anyhow::Context;
use chrono::Utc;
use crate::functions::utils::encrypt_data;

pub fn edit_note(conn: &Connection, key: &Key<Aes256Gcm>, provided_id: Option<i32>) -> anyhow::Result<()> {
    let id = match provided_id {
        Some(id) => id,
        None => {
            println!("Masukkan ID catatan yang akan diedit: ");
            let mut id = String::new();
            std::io::stdin().read_line(&mut id)?;
            id.trim().parse().unwrap_or(0)
        }
    };

    println!("\nMasukkan catatan baru (max 255 char, kosongkan untuk tidak mengubah): ");
    let mut note = String::new();
    std::io::stdin().read_line(&mut note)?;
    let note = note.trim();

    println!("Masukkan prioritas baru (1: Tinggi, 2: Sedang, 3: Rendah, 0: Tidak ubah): ");
    let mut prio_choice = String::new();
    std::io::stdin().read_line(&mut prio_choice)?;
    let prio_choice: i32 = prio_choice.trim().parse().unwrap_or(0);

    let priority = match prio_choice {
        1 => Some("Tinggi"),
        2 => Some("Sedang"),
        3 => Some("Rendah"),
        0 => None,
        _ => {
            println!("Pilihan tidak valid, prioritas tidak diubah.");
            None
        }
    };

    // Format datetime tanpa nanodetik
    let modified_at = Utc::now().to_rfc3339();

    if !note.is_empty() && priority.is_some() {
        let encrypted_note = encrypt_data(note, key)?;
        let encrypted_priority = encrypt_data(priority.unwrap(), key)?;
        conn.execute(
            "UPDATE notes SET note = ?1, priority = ?2, modifiedAt = ?3 WHERE id = ?4",
            params![encrypted_note, encrypted_priority, modified_at, id],
        ).context("Failed to update note and priority")?;
    } else if !note.is_empty() {
        let encrypted_note = encrypt_data(note, key)?;
        conn.execute(
            "UPDATE notes SET note = ?1, modifiedAt = ?2 WHERE id = ?3",
            params![encrypted_note, modified_at, id],
        ).context("Failed to update note")?;
    } else if let Some(p) = priority {
        let encrypted_priority = encrypt_data(p, key)?;
        conn.execute(
            "UPDATE notes SET priority = ?1, modifiedAt = ?2 WHERE id = ?3",
            params![encrypted_priority, modified_at, id],
        ).context("Failed to update priority")?;
    } else {
        println!("Tidak ada perubahan yang dibuat.");
        return Ok(());
    }
    println!("Catatan dengan ID {} berhasil diperbarui!", id);
    Ok(())
}