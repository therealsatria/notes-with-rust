use rusqlite::{Connection, params}; // Tambah params untuk parameter SQL
use textwrap::wrap;
use std::io;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use dotenv::dotenv;
use std::env;
use anyhow::Context;

// Struktur untuk merepresentasikan catatan
struct Note {
    id: i32,
    note: String,
    priority: String,
}

// Fungsi untuk enkripsi data
fn encrypt_data(data: &str, key: &Key<Aes256Gcm>) -> anyhow::Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // Nonce acak 12 byte
    let ciphertext = cipher.encrypt(&nonce, data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
    let mut encrypted = nonce.to_vec();
    encrypted.extend_from_slice(&ciphertext);
    Ok(encrypted)
}

// Fungsi untuk dekripsi data
fn decrypt_data(encrypted: &[u8], key: &Key<Aes256Gcm>) -> anyhow::Result<String> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted[0..12]); // Ambil 12 byte pertama sebagai nonce
    let ciphertext = &encrypted[12..];
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
    String::from_utf8(plaintext)
        .map_err(|e| anyhow::anyhow!("UTF-8 conversion failed: {:?}", e))
}

// Inisialisasi database
fn init_db() -> anyhow::Result<Connection> {
    let conn = Connection::open("notes.db")
        .context("Failed to open database")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note BLOB NOT NULL,
            priority BLOB NOT NULL
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

    conn.execute(
        "INSERT INTO notes (note, priority) VALUES (?1, ?2)",
        params![encrypted_note, encrypted_priority],
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
        |row| -> Result<Note, rusqlite::Error> {
            let encrypted_note: Vec<u8> = row.get(1)?;
            let encrypted_priority: Vec<u8> = row.get(2)?;
            let note = decrypt_data(&encrypted_note, key)
                .map_err(|_| rusqlite::Error::InvalidParameterName("Failed to decrypt note".into()))?;
            let priority = decrypt_data(&encrypted_priority, key)
                .map_err(|_| rusqlite::Error::InvalidParameterName("Failed to decrypt priority".into()))?;
            Ok(Note {
                id: row.get(0)?,
                note,
                priority,
            })
        }
    ).context("Failed to query notes")?;

    println!("\nDaftar Catatan:");
    println!("| {:<4} | {:<60} | {:<10} |", "ID", "Note", "Priority");
    println!("|------|--------------------------------------------------------------|------------|");

    for note in note_iter {
        let note = note?;
        let wrapped_note = wrap(&note.note, 60); // Perbaiki ¬e.note menjadi &note.note
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
fn edit_note(conn: &Connection, key: &Key<Aes256Gcm>) -> anyhow::Result<()> {
    println!("Masukkan ID catatan yang akan diedit: ");
    let mut id = String::new();
    io::stdin().read_line(&mut id)?;
    let id: i32 = id.trim().parse().unwrap_or(0);

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

    if !note.is_empty() && priority.is_some() {
        let encrypted_note = encrypt_data(note, key)?;
        let encrypted_priority = encrypt_data(priority.unwrap(), key)?;
        conn.execute(
            "UPDATE notes SET note = ?1, priority = ?2 WHERE id = ?3",
            params![encrypted_note, encrypted_priority, id],
        ).context("Failed to update note and priority")?;
    } else if !note.is_empty() {
        let encrypted_note = encrypt_data(note, key)?;
        conn.execute(
            "UPDATE notes SET note = ?1 WHERE id = ?2",
            params![encrypted_note, id],
        ).context("Failed to update note")?;
    } else if let Some(p) = priority {
        let encrypted_priority = encrypt_data(p, key)?;
        conn.execute(
            "UPDATE notes SET priority = ?1 WHERE id = ?2",
            params![encrypted_priority, id],
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
    conn.execute(
        "UPDATE notes SET priority = ?1 WHERE id = ?2",
        params![encrypted_priority, id],
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

    let mut stmt = conn.prepare("SELECT id, note, priority FROM notes WHERE id = ?1")
        .context("Failed to prepare statement")?;
    let mut note_iter = stmt.query_and_then(params![id], |row| -> rusqlite::Result<Note> {
        let id = row.get(0)?;
        let encrypted_note: Vec<u8> = row.get(1)?;
        let encrypted_priority: Vec<u8> = row.get(2)?;
        let note = decrypt_data(&encrypted_note, key)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let priority = decrypt_data(&encrypted_priority, key)
            .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        Ok(Note {
            id,
            note,
            priority,
        })
    }).context("Failed to query note by ID")?;

    if let Some(note) = note_iter.next() {
        let note = note?;
        println!("\nDetail Catatan:");
        println!("ID       : {}", note.id);
        println!("Catatan  : {}", note.note);
        println!("Prioritas: {}", note.priority);
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
            1 => edit_note(conn, key)?,
            2 => delete_note(conn)?,
            3 => change_priority(conn, key, id)?,
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

// Fungsi utama
fn main() -> anyhow::Result<()> {
    dotenv().ok(); // Muat file .env, abaikan jika tidak ada

    let encryption_key = env::var("ENCRYPTION_KEY")
        .context("ENCRYPTION_KEY harus diset di file .env")?;
    if encryption_key.len() != 32 {
        anyhow::bail!("ENCRYPTION_KEY harus tepat 32 byte untuk AES-256-GCM");
    }
    let key = Key::<Aes256Gcm>::from_slice(encryption_key.as_bytes());

    let conn = init_db()?;

    loop {
        show_notes(&conn, key)?;
        println!("\nSimple Notes App");
        println!("1. Tambah Catatan");
        println!("2. Tampilkan Catatan");
        println!("3. Hapus Catatan");
        println!("4. Edit Catatan");
        println!("5. Refresh Data");
        println!("6. Lihat Catatan Berdasarkan ID");
        println!("7. Keluar");
        println!("Pilih opsi (1-7): ");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice: i32 = choice.trim().parse().unwrap_or(0);

        match choice {
            1 => add_note(&conn, key)?,
            2 => show_notes(&conn, key)?,
            3 => delete_note(&conn)?,
            4 => edit_note(&conn, key)?,
            5 => refresh_data(&conn, key)?,
            6 => view_note_by_id(&conn, key)?,
            7 => {
                println!("Keluar dari aplikasi.");
                break;
            }
            _ => println!("Pilihan tidak valid!"),
        }
    }
    Ok(())
}