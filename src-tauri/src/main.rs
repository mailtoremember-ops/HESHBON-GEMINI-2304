#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager; // הוספנו כדי לשלוט בחלון
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct Entry {
    id: u32,
    content: String,
    #[serde(rename = "type")]
    entry_type: String,
    date: u64,
}

#[tauri::command]
fn check_auth(password: String) -> bool {
    password == "1234"
}

#[tauri::command]
fn get_entries() -> Vec<Entry> {
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
        .setup(|app| {
            // פותח את כלי המפתחים (הקונסולה) אוטומטית כשהאפליקציה עולה
            let window = app.get_window("main").unwrap();
            window.open_devtools();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![check_auth, get_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
