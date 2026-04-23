#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct Entry {
    id: u32,
    content: String,
    #[serde(rename = "type")] // משנים את השם כי type זו מילה שמורה
    entry_type: String,
    date: u64,
}

#[tauri::command]
fn check_auth(password: String) -> bool {
    // זמני: מדמה הצפנה
    password == "1234"
}

#[tauri::command]
fn get_entries() -> Vec<Entry> {
    // זמני: מוודא שה-Rust שולח נתונים ל-JS
    vec![
        Entry {
            id: 1,
            content: "החיבור ל-Rust עובד! זה נתון אמיתי שמגיע מהבקאנד ולא מה-Mock.".to_string(),
            entry_type: "positive".to_string(),
            date: 1680000000000,
        }
    ]
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![check_auth, get_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}