use rusqlite::{Connection, Result};
use textwrap::wrap;
use std::io;

// Struktur untuk merepresentasikan catatan
struct Note {
    id: i32,
    note: String,
    priority: String,
}

// Inisialisasi database
fn init_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open("notes.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note TEXT NOT NULL,
            priority TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

// Tambah catatan baru
fn add_note(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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

    conn.execute(
        "INSERT INTO notes (note, priority) VALUES (?1, ?2)",
        [note, priority], // Gunakan &str secara langsung
    )?;
    println!("Catatan berhasil ditambahkan!");
    Ok(())
}

// Tampilkan semua catatan
fn show_notes(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(
        "SELECT id, note, priority FROM notes ORDER BY CASE priority 
         WHEN 'Tinggi' THEN 1 WHEN 'Sedang' THEN 2 WHEN 'Rendah' THEN 3 END"
    )?;
    let note_iter = stmt.query_map([], |row| {
        Ok(Note {
            id: row.get(0)?,
            note: row.get(1)?,
            priority: row.get(2)?,
        })
    })?;

    println!("\nDaftar Catatan:");
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

// Hapus catatan berdasarkan ID
fn delete_note(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    println!("Masukkan ID catatan yang akan dihapus: ");
    let mut id = String::new();
    io::stdin().read_line(&mut id)?;
    let id: i32 = id.trim().parse().unwrap_or(0);

    conn.execute("DELETE FROM notes WHERE id = ?1", [&id])?;
    println!("Catatan dengan ID {} berhasil dihapus!", id);
    Ok(())
}

// Edit catatan berdasarkan ID
fn edit_note(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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
        conn.execute(
            "UPDATE notes SET note = ?1, priority = ?2 WHERE id = ?3",
            [note, priority.unwrap(), id.to_string().as_str()],
        )?;
    } else if !note.is_empty() {
        conn.execute(
            "UPDATE notes SET note = ?1 WHERE id = ?2",
            [note, id.to_string().as_str()],
        )?;
    } else if let Some(p) = priority {
        conn.execute(
            "UPDATE notes SET priority = ?1 WHERE id = ?2",
            [p, id.to_string().as_str()],
        )?;
    } else {
        println!("Tidak ada perubahan yang dibuat.");
        return Ok(());
    }
    println!("Catatan dengan ID {} berhasil diperbarui!", id);
    Ok(())
}

// Refresh data
fn refresh_data(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    show_notes(conn)?;
    println!("Data telah diperbarui.");
    Ok(())
}

// Fungsi utama
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = init_db()?;

    loop {
        show_notes(&conn)?;
        println!("\nSimple Notes App");
        println!("1. Tambah Catatan");
        println!("2. Tampilkan Catatan");
        println!("3. Hapus Catatan");
        println!("4. Edit Catatan");
        println!("5. Refresh Data");
        println!("6. Keluar");
        println!("Pilih opsi (1-6): ");

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice: i32 = choice.trim().parse().unwrap_or(0);

        match choice {
            1 => add_note(&conn)?,
            2 => show_notes(&conn)?,
            3 => delete_note(&conn)?,
            4 => edit_note(&conn)?,
            5 => refresh_data(&conn)?,
            6 => {
                println!("Keluar dari aplikasi.");
                break;
            }
            _ => println!("Pilihan tidak valid!"),
        }
    }
    Ok(())
}