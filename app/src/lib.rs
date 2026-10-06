mod commands;
mod state;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_window_event(tray::on_window_event)
        .setup(|app| {
            // Mitgelieferte ffmpeg-Sidecars liegen neben der ausführbaren Datei.
            let search_dirs: Vec<_> = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
                .into_iter()
                .collect();
            let tools = audioconv_core::Tools::locate(&search_dirs);
            let user_presets = app.path().app_config_dir().ok().map(|dir| dir.join("presets.json"));
            app.manage(state::AppState::new(tools, user_presets));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::environment,
            commands::presets,
            commands::save_preset,
            commands::delete_preset,
            commands::import_presets,
            commands::read_text_file,
            commands::write_text_file,
            commands::add_paths,
            commands::remove_items,
            commands::start,
            commands::cancel,
            commands::cancel_all,
            tray::set_minimize_to_tray,
            commands::install_info,
        ])
        .build(tauri::generate_context!())
        .expect("Anwendung konnte nicht gestartet werden")
        .run(|app, event| {
            // macOS: Klick aufs Dock-Symbol holt ein verstecktes Fenster zurück.
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                tray::show_main(app);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}
