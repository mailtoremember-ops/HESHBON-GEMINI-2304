#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{Manager, State, AppHandle};
use rusqlite::{Connection, params, OptionalExtension};
use std::sync::Mutex;
use std::fs;
use serde::{Serialize, Deserialize};

// --- ספריות הצפנה ---
use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use argon2::Argon2;
use rand::{RngCore, rngs::OsRng};

// --- ניהול זיכרון (State) ---
// כאן יישמר המפתח (32 בתים) בזמן שהאפליקציה פתוחה. כשהיא נסגרת, זה נמחק.
struct AppState {
    encryption_key: Mutex<Option<[u8; 32]>>,
}

#[derive(Serialize, Deserialize)]
struct EntryResponse {
    id: i32,
    timestamp: u64,
    payload: String, // ה-JSON המפוענח שיועבר לדפדפן
}

// --- פונקציות עזר: מסד נתונים ---
fn get_db_path(app_handle: &AppHandle) -> String {
    let local_data_dir = app_handle.path_resolver().app_local_data_dir().unwrap();
    fs::create_dir_all(&local_data_dir).unwrap(); // מוודא שהתיקייה קיימת
    local_data_dir.join("journal.sqlite").to_str().unwrap().to_string()
}

fn init_db(app_handle: &AppHandle) {
    let db_path = get_db_path(app_handle);
    let conn = Connection::open(db_path).unwrap();

    // טבלת הגדרות (גלוי) - שומרת את ה-Salt ואת הבלוק לאימות הסיסמה
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value BLOB
        )",
        [],
    ).unwrap();

    // טבלת רשומות - timestamp גלוי לסינון מהיר, payload מוצפן
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp INTEGER NOT NULL,
            payload BLOB NOT NULL
        )",
        [],
    ).unwrap();
}

// --- פונקציות עזר: הצפנה ---
fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt, &mut key).expect("Argon2 failed");
    key
}

fn encrypt_data(key: &[u8; 32], plaintext: &str) -> Vec<u8> {
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).expect("Encryption failed");
    
    // משרשרים את ה-Nonce (12 בתים) לפני המידע המוצפן כדי שנוכל לפענח אחר כך
    let mut final_payload = nonce_bytes.to_vec();
    final_payload.extend(ciphertext);
    final_payload
}

fn decrypt_data(key: &[u8; 32], encrypted_data: &[u8]) -> Result<String, ()> {
    if encrypted_data.len() < 12 { return Err(()); }
    
    let cipher = Aes256Gcm::new(aes_gcm::Key::<Aes256Gcm>::from_slice(key));
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    match cipher.decrypt(nonce, ciphertext) {
        Ok(plaintext_bytes) => Ok(String::from_utf8(plaintext_bytes).unwrap()),
        Err(_) => Err(()),
    }
}

// --- פקודות Tauri שמקושרות ל-JavaScript ---

#[tauri::command]
fn check_is_setup(app_handle: AppHandle) -> bool {
    let conn = Connection::open(get_db_path(&app_handle)).unwrap();
    let salt: Option<Vec<u8>> = conn.query_row("SELECT value FROM app_settings WHERE key = 'auth_salt'", [], |row| row.get(0)).optional().unwrap();
    salt.is_some()
}

#[tauri::command]
fn setup_password(app_handle: AppHandle, state: State<AppState>, password: String) -> bool {
    // מייצרים Salt אקראי
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    
    // מייצרים מפתח מהסיסמה
    let key = derive_key(&password, &salt);
    
    // מצפינים את המילה OK כדי לבדוק את הסיסמה בהתחברויות הבאות
    let verification_block = encrypt_data(&key, "OK");

    let conn = Connection::open(get_db_path(&app_handle)).unwrap();
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('auth_salt', ?1)", params![salt]).unwrap();
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('verify_block', ?1)", params![verification_block]).unwrap();

    // שומרים את המפתח בזיכרון
    *state.encryption_key.lock().unwrap() = Some(key);
    true
}

#[tauri::command]
fn login(app_handle: AppHandle, state: State<AppState>, password: String) -> bool {
    let conn = Connection::open(get_db_path(&app_handle)).unwrap();
    
    let salt: Vec<u8> = match conn.query_row("SELECT value FROM app_settings WHERE key = 'auth_salt'", [], |row| row.get(0)) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    let verify_block: Vec<u8> = match conn.query_row("SELECT value FROM app_settings WHERE key = 'verify_block'", [], |row| row.get(0)) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let key = derive_key(&password, &salt);
    
    // אם ההצפנה עובדת והצלחנו לפענח את המילה OK, הסיסמה נכונה!
    if let Ok(decrypted) = decrypt_data(&key, &verify_block) {
        if decrypted == "OK" {
            *state.encryption_key.lock().unwrap() = Some(key);
            return true;
        }
    }
    false
}

#[tauri::command]
fn save_entry(app_handle: AppHandle, state: State<AppState>, timestamp: u64, payload_json: String) -> Result<i32, String> {
    let key_guard = state.encryption_key.lock().unwrap();
    let key = key_guard.as_ref().ok_or("Not logged in")?;
    
    let encrypted_payload = encrypt_data(key, &payload_json);
    
    let conn = Connection::open(get_db_path(&app_handle)).unwrap();
    conn.execute("INSERT INTO entries (timestamp, payload) VALUES (?1, ?2)", params![timestamp, encrypted_payload]).map_err(|e| e.to_string())?;
    
    let id = conn.last_insert_rowid() as i32;
    Ok(id)
}

#[tauri::command]
fn get_entries(app_handle: AppHandle, state: State<AppState>) -> Result<Vec<EntryResponse>, String> {
    let key_guard = state.encryption_key.lock().unwrap();
    let key = key_guard.as_ref().ok_or("Not logged in")?;

    let conn = Connection::open(get_db_path(&app_handle)).unwrap();
    let mut stmt = conn.prepare("SELECT id, timestamp, payload FROM entries ORDER BY timestamp DESC").unwrap();
    
    let entry_iter = stmt.query_map([], |row| {
        let id: i32 = row.get(0)?;
        let timestamp: u64 = row.get(1)?;
        let payload_blob: Vec<u8> = row.get(2)?;
        Ok((id, timestamp, payload_blob))
    }).unwrap();

    let mut results = Vec::new();
    for entry in entry_iter {
        let (id, timestamp, payload_blob) = entry.unwrap();
        // מפענחים כל שורה. אם הצליח, מוסיפים לרשימה.
        if let Ok(decrypted_json) = decrypt_data(key, &payload_blob) {
            results.push(EntryResponse {
                id,
                timestamp,
                payload: decrypted_json,
            });
        }
    }
    Ok(results)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            encryption_key: Mutex::new(None),
        })
        .setup(|app| {
            init_db(&app.handle());
            app.get_window("main").unwrap().open_devtools(); // פותח קונסולה לבדיקות
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_is_setup,
            setup_password,
            login,
            save_entry,
            get_entries
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
