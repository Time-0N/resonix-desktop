use tauri::Manager;

mod audio;
pub mod tauri_commands;
pub mod db;
pub mod library;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // create Send+Sync manager that only holds atomics + command sender
            let mgr = tauri_commands::audio::AudioManager::new(&app.handle());
            app.manage(mgr); // this is now Send + Sync, OK

            let pool = db::init_db().map_err(|e| anyhow::anyhow!(e))?;
            app.manage(pool);

            #[cfg(debug_assertions)]
            {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            tauri_commands::audio::load_audio_file,
            tauri_commands::audio::set_queue,
            tauri_commands::audio::play_audio,
            tauri_commands::audio::pause_audio,
            tauri_commands::audio::stop_audio,
            tauri_commands::audio::set_volume,
            tauri_commands::audio::get_duration,
            tauri_commands::audio::get_position,
            tauri_commands::audio::seek_to,
            tauri_commands::audio::next_track,
            tauri_commands::audio::prev_track,
            tauri_commands::audio::play_selection,

            // --- library ---
            tauri_commands::library::choose_library_dir,
            tauri_commands::library::scan_library,
            tauri_commands::library::get_cover_art,
            tauri_commands::library::get_cover_thumb,

            // --- settings ---
            tauri_commands::settings::get_settings,
            tauri_commands::settings::set_library_root,
            tauri_commands::settings::set_use_managed_dir,
            tauri_commands::settings::set_managed_root,

            // --- tracks ---
            tauri_commands::library::list_tracks,
            tauri_commands::library::list_unregistered,
            tauri_commands::library::list_artists,
            tauri_commands::library::list_albums,
            tauri_commands::library::pick_audio_file,

            // --- ingestion ---
            tauri_commands::ingestion::register_artist,
            tauri_commands::ingestion::register_album,
            tauri_commands::ingestion::register_track,

        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
