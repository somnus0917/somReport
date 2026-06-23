pub mod capture;
pub mod commands;
pub mod domain;
pub mod pipeline;
pub mod platform;
pub mod providers;
pub mod reporting;
pub mod storage;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let db_path = platform::paths::db_path();
            platform::paths::ensure_dirs().expect("failed to create app directories");

            let db = storage::Database::new(&db_path).expect("failed to open database");
            app.manage(db);

            let settings = {
                let db_ref = app.state::<storage::Database>();
                db_ref.get_settings().unwrap_or_default()
            };

            let scheduler = pipeline::scheduler::CaptureScheduler::new(
                settings.capture_interval_secs as u64,
                settings.idle_timeout_secs as u64,
            );
            app.manage(scheduler);

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            platform::tray::create_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_today,
            commands::update_activity,
            commands::delete_activity,
            commands::generate_report,
            commands::list_reports,
            commands::start_recording,
            commands::pause_recording,
            commands::stop_recording,
            commands::get_recording_state,
            commands::save_provider_key,
            commands::test_provider_key,
            commands::get_settings,
            commands::save_settings,
            commands::clear_all_data,
            commands::get_daily_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
