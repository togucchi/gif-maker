mod commands;
mod encode;
mod ffmpeg;
mod media;
mod probe;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState::default())
        .setup(|app| {
            // Thumbnails, proxies and preview GIFs live in the cache dir.
            let cache = app.path().app_cache_dir()?;
            std::fs::create_dir_all(&cache)?;
            app.asset_protocol_scope().allow_directory(&cache, true)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::probe_video,
            commands::generate_thumbnails,
            commands::prepare_proxy,
            commands::estimate_size,
            commands::encode_gif,
            commands::cancel_encode,
            commands::save_gif,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GIF Maker");
}
