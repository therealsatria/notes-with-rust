use rusqlite::{Connection, params}; // Hapus 'batch'
use textwrap::wrap;
use std::io;
use aes_gcm::{
    Aes256Gcm, Key
};
use dotenv::dotenv;
use std::env;
use anyhow::Context;
use chrono::{DateTime, Utc};
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::File;
mod cipher;
use cipher::encrypt_data;
use cipher::decrypt_data;

// Struktur untuk merepresentasikan catatan
struct Note {
    id: i32,
    note: String,
    priority: String,
    created_at: DateTime<Utc>,
    modified_at: Option<DateTime<Utc>>,
}

// Inisialisasi database
fn init_db() -> anyhow::Result<Connection> {
    let conn = Connection::open("notes.db")
        .context("Failed to open database")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note BLOB NOT NULL,
            priority BLOB NOT NULL,
            createdAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            modifiedAt DATETIME
        )",
        [],
    ).context("Failed to create table")?;
    Ok(conn)
}

// Tambah catatan baru
fn add_note(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan catatan (max 255 char): ");
    let mut note = String::new();
    io::stdin().read_line(&mut note)?;
    let note = note.trim();

    println!("Masukkan prioritas (1: Tinggi, 2: Sedang, 3: Rendah): ");
    let mut prio_choice = String::new();
    io::stdin().read_line(&mut prio_choice)?;
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

// Tampilkan semua catatan
fn show_notes(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, note, priority FROM notes ORDER BY CASE priority 
         WHEN ?1 THEN 1 WHEN ?2 THEN 2 WHEN ?3 THEN 3 END"
    ).context("Failed to prepare statement")?;
    
    let encrypted_tinggi = encrypt_data("Tinggi", key)?;
    let encrypted_sedang = encrypt_data("Sedang", key)?;
    let encrypted_rendah = encrypt_data("Rendah", key)?;

    let note_iter = stmt.query_and_then(
        params![encrypted_tinggi, encrypted_sedang, encrypted_rendah],
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
                created_at: Utc::now(), // Dummy
                modified_at: None, // Dummy
            })
        }
    ).context("Failed to query notes")?;

    println!("\nDaftar Catatan:");
    println!("| {:<4} | {:<60} | {:<10} |", "ID", "Note", "Priority");
    println!("|------|--------------------------------------------------------------|------------|");

    for note in note_iter {
        let note = note?;
        let wrapped_note = wrap(&note

            .note, 60);
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

// Hapus catatan berdasarkan ID
fn delete_note(conn: &Connection) -> anyhow::Result<()> {
    println!("Masukkan ID catatan yang akan dihapus: ");
    let mut id = String::new();
    io::stdin().read_line(&mut id)?;
    let id: i32 = id.trim().parse().unwrap_or(0);

    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .context("Failed to delete note")?;
    println!("Catatan dengan ID {} berhasil dihapus!", id);
    Ok(())
}

// Edit catatan berdasarkan ID
fn edit_note(conn: &Connection, key: &Key<Aes256Gcm>, provided_id: Option<i32>) -> anyhow::Result<()> {
    let id = match provided_id {
        Some(id) => id,
        None => {
            println!("Masukkan ID catatan yang akan diedit: ");
            let mut id = String::new();
            io::stdin().read_line(&mut id)?;
            id.trim().parse().unwrap_or(0)
        }
    };

    println!("\nMasukkan catatan baru (max 255 char, kosongkan untuk tidak mengubah): ");
    let mut note = String::new();
    io::stdin().read_line(&mut note)?;
    let note = note.trim();

    println!("Masukkan prioritas baru (1: Tinggi, 2: Sedang, 3: Rendah, 0: Tidak ubah): ");
    let mut prio_choice = String::new();
    io::stdin().read_line(&mut prio_choice)?;
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

// Ganti prioritas catatan berdasarkan ID
fn change_priority(conn: &Connection, key: &Key<Aes256Gcm>, id: i32) -> anyhow::Result<()> {
    println!("Masukkan prioritas baru (1: Tinggi, 2: Sedang, 3: Rendah): ");
    let mut prio_choice = String::new();
    io::stdin().read_line(&mut prio_choice)?;
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

    let encrypted_priority = encrypt_data(priority, key)?;
    let modified_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE notes SET priority = ?1, modifiedAt = ?2 WHERE id = ?3",
        params![encrypted_priority, modified_at, id],
    ).context("Failed to change priority")?;
    println!("Prioritas catatan dengan ID {} berhasil diperbarui!", id);
    Ok(())
}

// Tampilan halaman khusus untuk melihat note berdasarkan ID
fn view_note_by_id(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan ID catatan yang ingin dilihat: ");
    let mut id = String::new();
    io::stdin().read_line(&mut id)?;
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
        io::stdin().read_line(&mut choice)?;
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

// Refresh data
fn refresh_data(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    show_notes(conn, key)?;
    println!("Data telah diperbarui.");
    Ok(())
}

// Export semua data ke CSV (unencrypted)
fn export_to_csv(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
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

// Import data dari CSV (encrypt sebelum simpan)
fn import_from_csv(conn: &mut Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> { // Ubah ke &mut Connection
    println!("Masukkan path file CSV untuk diimpor (default: 'notes_import.csv'): ");
    let mut path = String::new();
    io::stdin().read_line(&mut path)?;
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

        let encrypted_note = encrypt_data(&note

            , key)?;
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

// Pencarian catatan berdasarkan note
fn search_notes(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan kata kunci untuk mencari catatan: ");
    let mut keyword = String::new();
    io::stdin().read_line(&mut keyword)?;
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
                created_at: Utc::now(), // Dummy
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


//wrap(&note.note, 60);
// Fungsi utama
fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let encryption_key = env::var("ENCRYPTION_KEY")
        .context("ENCRYPTION_KEY harus diset di file .env")?;
    if encryption_key.len() != 32 {
        anyhow::bail!("ENCRYPTION_KEY harus tepat 32 byte untuk AES-256-GCM");
    }
    let key = Key::<Aes256Gcm>::from_slice(encryption_key.as_bytes());

    let mut conn = init_db()?;

    loop {
        //show_notes(&conn, key)?;
        println!("\nSimple Notes App");
        println!("1. Tambah Catatan");
        println!("2. Tampilkan Catatan");
        println!("3. Hapus Catatan");
        println!("4. Edit Catatan");
        println!("5. Refresh Data");
        println!("6. Lihat Catatan Berdasarkan ID");
        println!("7. Export ke CSV");
        println!("8. Import dari CSV");
        println!("9. Search Catatan"); // Tambah opsi baru
        println!("10. Keluar");
        println!("Pilih opsi (1-10): ");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice: i32 = choice.trim().parse().unwrap_or(0);

        match choice {
            1 => add_note(&conn, key)?,
            2 => show_notes(&conn, key)?,
            3 => delete_note(&conn)?,
            4 => edit_note(&conn, key, None)?,
            5 => refresh_data(&conn, key)?,
            6 => view_note_by_id(&conn, key)?,
            7 => export_to_csv(&conn, key)?,
            8 => import_from_csv(&mut conn, key)?,
            9 => search_notes(&conn, key)?, // Panggil fungsi pencarian
            10 => {
                println!("Keluar dari aplikasi.");
                break;
            }
            _ => println!("Pilihan tidak valid!"),
        }
    }
    Ok(())
}