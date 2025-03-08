use anyhow::Context;
use dotenv::dotenv;
use std::env;

// Deklarasi modul

mod functions {
    pub mod add_note;
    pub mod delete_note;
    pub mod edit_note;
    pub mod export_to_csv;
    pub mod import_from_csv;
    pub mod init_db;
    pub mod refresh_data;
    pub mod search_notes;
    pub mod show_notes;
    pub mod utils;
    pub mod view_note_by_id;
}

// Impor fungsi dari modul
use functions::add_note::add_note;
use functions::delete_note::delete_note;
use functions::edit_note::edit_note;
use functions::export_to_csv::export_to_csv;
use functions::import_from_csv::import_from_csv;
use functions::init_db::init_db;
use functions::refresh_data::refresh_data;
use functions::search_notes::search_notes;
use functions::show_notes::show_notes;
use functions::view_note_by_id::view_note_by_id;

fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let encryption_key = env::var("ENCRYPTION_KEY")
        .context("ENCRYPTION_KEY harus diset di file .env")?;
    if encryption_key.len() != 32 {
        anyhow::bail!("ENCRYPTION_KEY harus tepat 32 byte untuk AES-256-GCM");
    }
    let key = aes_gcm::Key::<aes_gcm::Aes256Gcm>::from_slice(encryption_key.as_bytes());

    let mut conn = init_db()?;

    loop {
        println!("\nSimple Notes App");
        println!("1. Tambah Catatan");
        println!("2. Tampilkan Catatan");
        println!("3. Hapus Catatan");
        println!("4. Edit Catatan");
        println!("5. Refresh Data");
        println!("6. Lihat Catatan Berdasarkan ID");
        println!("7. Export ke CSV");
        println!("8. Import dari CSV");
        println!("9. Search Catatan");
        println!("10. Keluar");
        println!("Pilih opsi (1-10): ");

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice)?;
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
            9 => search_notes(&conn, key)?,
            10 => {
                println!("Keluar dari aplikasi.");
                break;
            }
            _ => println!("Pilihan tidak valid!"),
        }
    }
    Ok(())
}