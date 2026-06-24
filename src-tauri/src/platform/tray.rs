use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};

use crate::pipeline::scheduler::CaptureScheduler;
use crate::platform::notifications;

const TOGGLE_ID: &str = "toggle_recording";
const OPEN_ID: &str = "open_som_report";
const QUIT_ID: &str = "quit";

pub fn create_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let is_recording = {
        let scheduler = app.state::<CaptureScheduler>();
        scheduler.state() == crate::pipeline::scheduler::RecordingState::Recording
    };

    let toggle = CheckMenuItem::with_id(
        app,
        TOGGLE_ID,
        "Recording",
        true,
        is_recording,
        None::<&str>,
    )?;
    let open = MenuItem::with_id(app, OPEN_ID, "打开日报助手", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "退出", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&toggle, &open, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("日报助手")
        .on_menu_event(move |app, event| {
            let id = event.id().as_ref();
            match id {
                TOGGLE_ID => {
                    let scheduler = app.state::<CaptureScheduler>();
                    let was_recording =
                        scheduler.state() == crate::pipeline::scheduler::RecordingState::Recording;
                    if was_recording {
                        scheduler.pause();
                        let _ = app.emit("recording-status", "paused");
                        notifications::notify("日报助手", "录制已暂停");
                    } else {
                        scheduler.start();
                        let _ = app.emit("recording-status", "recording");
                        notifications::notify("日报助手", "录制已开始");
                    }
                }
                OPEN_ID => {
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
                QUIT_ID => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
