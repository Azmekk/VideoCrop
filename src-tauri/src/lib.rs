mod commands;
mod extensions;

use extensions::builder_extensions::BuilderExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .register_all_handlers()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
