use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use chrono::{DateTime, Utc};

// Struktur untuk merepresentasikan catatan
pub struct Note {
    pub id: i32,
    pub note: String,
    pub priority: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
}

// Fungsi untuk enkripsi data
pub fn encrypt_data(data: &str, key: &Key<Aes256Gcm>) -> anyhow::Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
    let mut encrypted = nonce.to_vec();
    encrypted.extend_from_slice(&ciphertext);
    Ok(encrypted)
}

// Fungsi untuk dekripsi data
pub fn decrypt_data(encrypted: &[u8], key: &Key<Aes256Gcm>) -> anyhow::Result<String> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&encrypted[0..12]);
    let ciphertext = &encrypted[12..];
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
    String::from_utf8(plaintext)
        .map_err(|e| anyhow::anyhow!("UTF-8 conversion failed: {:?}", e))
}