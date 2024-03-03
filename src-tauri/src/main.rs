// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "linux"))]
use tauri_plugin_dialog::DialogExt;

#[cfg(target_os = "linux")]
#[tauri::command]
async fn get_files(window: tauri::Window) -> Vec<String> {
    let files = rfd::AsyncFileDialog::new()
        .add_filter("Image", &["png", "jpeg", "jpg", "webp", "bmp", "tiff", "gif"])
        .set_parent(&window)
        .pick_files()
        .await;

    let mut files_paths: Vec<String> = Vec::new();

    if files.is_some() {
        for file_handle in &files.unwrap() {
            let path = file_handle.path().to_str();
            if path.is_some() {
                files_paths.push(path.unwrap().to_string())
            }
        }
    }
    files_paths
}

#[cfg(not(target_os = "linux"))]
#[tauri::command]
async fn get_files(window: tauri::Window) -> Vec<String> {
    let files = window
        .dialog()
        .file()
        .add_filter("Image", &["png", "jpeg", "jpg", "webp", "bmp", "tiff", "gif"])
        .blocking_pick_files();

    let mut files_paths: Vec<String> = Vec::new();

    if files.is_some() {
        for file in &files.unwrap() {
            let path = file.path.to_str();
            if path.is_some() {
                files_paths.push(path.unwrap().to_string())
            }
        }
    }
    files_paths
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .invoke_handler(tauri::generate_handler![get_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
