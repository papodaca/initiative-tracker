// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
async fn get_files_impl(window: tauri::Window) -> Option<Vec<String>> {
    let files = rfd::AsyncFileDialog::new()
        .add_filter("image", &["png", "jpeg", "jpg", "webp", "bmp", "tiff", "gif"])
        .set_parent(&window)
        .pick_files()
        .await;

    let mut files_paths: Vec<String> = Vec::new();

    if files.is_none() {
        Some(files_paths)
    } else {
        for file_handle in &files.unwrap() {
            files_paths.push(file_handle.path().to_str()?.to_string())
        }

        Some(files_paths)
    }
}

#[cfg(not(target_os = "linux"))]
async fn get_files_impl(window: tauri::Window) -> Option<Vec<String>> {
    None;
}

#[tauri::command]
async fn get_files(window: tauri::Window) -> Result<Vec<String>, ()> {
    let files = get_files_impl(window).await;
    if files.is_none() {
        Err(())
    } else {
        Ok(files.expect(""))
    }
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
