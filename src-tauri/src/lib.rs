use std::{path::PathBuf, sync::{Arc, Mutex}, time::Duration};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State, WindowEvent,
};
use tauri_plugin_store::StoreExt;
use tokio::time::sleep;

use chrono::Utc;

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;

#[derive(Default)]
struct AppState {
    auto_enabled: Arc<std::sync::atomic::AtomicBool>,
    apply_all: Arc<std::sync::atomic::AtomicBool>,
    last_status: Mutex<SyncStatus>,
}

#[derive(Debug, Default, Clone, Serialize)]
struct SyncStatus {
    last_url: Option<String>,
    last_saved_path: Option<String>,
    last_result: Option<String>,
    last_error: Option<String>,
    last_run: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    auto_enabled: bool,
    apply_all: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_enabled: true,
            apply_all: true,
        }
    }
}

#[tauri::command]
fn sync_wallpaper(apply_all: bool, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<SyncStatus, String> {
    perform_sync(&app, &state, apply_all)
}

#[tauri::command]
fn set_auto_sync(enabled: bool, apply_all: bool, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    state.auto_enabled.store(enabled, std::sync::atomic::Ordering::SeqCst);
    state.apply_all.store(apply_all, std::sync::atomic::Ordering::SeqCst);
    
    // Persist to store
    if let Ok(store) = app.store("settings.json") {
        let settings = AppSettings { auto_enabled: enabled, apply_all };
        let _ = store.set("app_settings", serde_json::to_value(settings).unwrap_or_default());
        let _ = store.save();
    }
    
    Ok(())
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(AppSettings {
        auto_enabled: state.auto_enabled.load(std::sync::atomic::Ordering::SeqCst),
        apply_all: state.apply_all.load(std::sync::atomic::Ordering::SeqCst),
    })
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> Result<SyncStatus, String> {
    Ok(state
        .last_status
        .lock()
        .map_err(|e| e.to_string())?
        .clone())
}

fn set_wallpaper(path: &PathBuf, apply_all: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let target = if apply_all {
            "set picture of every desktop to"
        } else {
            "set picture of desktop 1 to"
        };

        let script = format!(
            "tell application \"System Events\" to {target} POSIX file \"{}\"",
            path.display()
        );

        std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (path, apply_all);
        return Err("Setting wallpaper is implemented for macOS only".to_string());
    }
}

fn perform_sync(app: &tauri::AppHandle, state: &State<'_, AppState>, apply_all: bool) -> Result<SyncStatus, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let api = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";
    let response: serde_json::Value = client
        .get(api)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let image_path = response
        .get("images")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("url"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Unexpected Bing API response".to_string())?;

    let image_url = format!("https://www.bing.com{}", image_path);

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to locate cache dir: {e}"))?;

    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    let filename = image_url
        .split('/')
        .last()
        .unwrap_or("bing_wallpaper.jpg");
    let save_path = cache_dir.join(filename);

    let bytes = client
        .get(&image_url)
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;

    std::fs::write(&save_path, bytes).map_err(|e| e.to_string())?;

    set_wallpaper(&save_path, apply_all).map_err(|e| e.to_string())?;

    let mut status = state.last_status.lock().map_err(|e| e.to_string())?.clone();
    status.last_url = Some(image_url);
    status.last_saved_path = Some(save_path.display().to_string());
    status.last_result = Some(if apply_all {
        "Applied to all displays".to_string()
    } else {
        "Applied to main display".to_string()
    });
    status.last_error = None;
    status.last_run = Some(chrono::Utc::now().to_rfc3339());
    *state.last_status.lock().map_err(|e| e.to_string())? = status.clone();

    Ok(status)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::default();

    let app_state = state.auto_enabled.clone();
    let apply_all_state = state.apply_all.clone();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(ActivationPolicy::Accessory);
            }

            // Create tray icon
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Load settings from store
            if let Ok(store) = app.store("settings.json") {
                if let Some(settings_value) = store.get("app_settings") {
                    if let Ok(settings) = serde_json::from_value::<AppSettings>(settings_value.clone()) {
                        app_state.store(settings.auto_enabled, std::sync::atomic::Ordering::SeqCst);
                        apply_all_state.store(settings.apply_all, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            }

            let handle = app.handle().clone();
            let background_state = app_state.clone();
            let background_apply_all = apply_all_state.clone();

            // Initial sync on startup
            let initial_handle = handle.clone();
            let initial_apply_all = background_apply_all.load(std::sync::atomic::Ordering::SeqCst);
            std::thread::spawn(move || {
                let initial_state = initial_handle.state::<AppState>();
                let _ = perform_sync(&initial_handle, &initial_state, initial_apply_all);
            });

            tauri::async_runtime::spawn(async move {
                loop {
                    if background_state.load(std::sync::atomic::Ordering::SeqCst) {
                        let apply_all = background_apply_all.load(std::sync::atomic::Ordering::SeqCst);
                        let state = handle.state::<AppState>();
                        if let Err(err) = perform_sync(&handle, &state, apply_all) {
                            if let Ok(mut guard) = state.last_status.lock() {
                                guard.last_error = Some(err);
                                guard.last_run = Some(Utc::now().to_rfc3339());
                            }
                        }
                    }
                    sleep(Duration::from_secs(60 * 60)).await;
                }
            });
            Ok(())
        })
        .manage(state)
        .invoke_handler(tauri::generate_handler![sync_wallpaper, set_auto_sync, get_status, get_settings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
