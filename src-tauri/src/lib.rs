mod audio;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let audio_manager = audio::AudioManager::new();
    audio_manager.start_audio_thread();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(audio_manager)
        .invoke_handler(tauri::generate_handler![
            audio::load_audio_file,
            audio::play_audio,
            audio::pause_audio,
            audio::stop_audio,
            audio::set_volume,
            audio::get_duration,
            audio::get_position,
            audio::seek_to,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}