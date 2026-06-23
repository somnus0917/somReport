pub mod capture;
pub mod commands;
pub mod domain;
pub mod pipeline;
pub mod platform;
pub mod providers;
pub mod reporting;
pub mod storage;

use std::sync::Arc;

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
            let settings = db.get_settings().unwrap_or_default();
            app.manage(db.clone());

            let scheduler = pipeline::scheduler::CaptureScheduler::new(
                settings.capture_interval_secs as u64,
                settings.idle_timeout_secs as u64,
            );
            if settings.auto_start {
                scheduler.start();
            }

            let idle_detector =
                platform::idle::IdleDetector::new(settings.idle_timeout_secs as u64);
            let scheduler_task = scheduler.clone();
            let idle_task = idle_detector.clone();
            let idle_rx = idle_detector.idle_rx();
            let app_handle = app.handle().clone();
            let queue_worker = pipeline::queue::QueueWorker::new(Arc::new(db.clone()), 0.98);

            tauri::async_runtime::spawn(async move {
                idle_task.run().await;
            });
            tauri::async_runtime::spawn(async move {
                scheduler_task
                    .run(
                        app_handle,
                        Box::new(capture::X11CaptureProvider::new()),
                        queue_worker,
                        db,
                        idle_rx,
                    )
                    .await;
            });

            app.manage(scheduler);
            app.manage(idle_detector);

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
