use std::{path::PathBuf, sync::{Arc, Mutex}, time::Duration};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tauri::{
    image::Image,
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
    resolution: Mutex<String>,
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
    resolution: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_enabled: true,
            apply_all: true,
            resolution: "UHD".to_string(),
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
        let resolution = state.resolution.lock().map_err(|e| e.to_string())?;
        let settings = AppSettings {
            auto_enabled: enabled,
            apply_all,
            resolution: resolution.clone(),
        };
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
        resolution: state.resolution.lock().map_err(|e| e.to_string())?.clone(),
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

#[tauri::command]
fn set_resolution(resolution: String, state: State<'_, AppState>, app: tauri::AppHandle) -> Result<(), String> {
    *state.resolution.lock().map_err(|e| e.to_string())? = resolution.clone();
    
    // Persist to store
    if let Ok(store) = app.store("settings.json") {
        let settings = AppSettings {
            auto_enabled: state.auto_enabled.load(std::sync::atomic::Ordering::SeqCst),
            apply_all: state.apply_all.load(std::sync::atomic::Ordering::SeqCst),
            resolution,
        };
        let _ = store.set("app_settings", serde_json::to_value(settings).unwrap_or_default());
        let _ = store.save();
    }
    
    Ok(())
}

#[tauri::command]
fn clear_cache(app: tauri::AppHandle) -> Result<String, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to locate cache dir: {e}"))?;

    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
        Ok("Cache cleared successfully".to_string())
    } else {
        Ok("Cache directory does not exist".to_string())
    }
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

    // Get resolution setting and modify URL
    let resolution = state.resolution.lock().map_err(|e| e.to_string())?;
    let modified_path = if resolution.as_str() == "UHD" {
        // Replace the resolution in the path (e.g., _1920x1080.jpg -> _UHD.jpg)
        let re = regex::Regex::new(r"_\d+x\d+\.jpg").unwrap();
        re.replace(image_path, "_UHD.jpg").to_string()
    } else if resolution.as_str() == "1920x1080" {
        // Ensure it's using 1920x1080
        let re = regex::Regex::new(r"_\d+x\d+\.jpg").unwrap();
        re.replace(image_path, "_1920x1080.jpg").to_string()
    } else {
        image_path.to_string()
    };

    let image_url = format!("https://www.bing.com{}", modified_path);

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to locate cache dir: {e}"))?;

    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;

    // Extract clean filename from URL
    // URL format: /th?id=OHR.LlamaDay_EN-US5971354659_UHD.jpg&rf=...
    // We want: LlamaDay_EN-US5971354659_UHD.jpg or just use date-based naming
    let filename = if let Some(id_param) = modified_path.split("id=").nth(1) {
        if let Some(name_part) = id_param.split('&').next() {
            // Extract the part after "OHR."
            if let Some(clean_name) = name_part.strip_prefix("OHR.") {
                clean_name.to_string()
            } else {
                name_part.to_string()
            }
        } else {
            "bing_wallpaper.jpg".to_string()
        }
    } else {
        "bing_wallpaper.jpg".to_string()
    };
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

            // Load tray icon - use template image for macOS dark/light mode support
            #[cfg(target_os = "macos")]
            let tray_icon = {
                // On macOS, load the light icon and mark it as template for automatic dark/light adaptation
                let icon_path = app.path().resource_dir()?.join("icons/light-logo.png");
                let icon_bytes = std::fs::read(icon_path)?;
                let icon_image = image::load_from_memory(&icon_bytes)
                    .map_err(|e| tauri::Error::AssetNotFound(format!("Failed to load icon: {}", e)))?;
                let icon_rgba = icon_image.to_rgba8();
                let (width, height) = icon_rgba.dimensions();
                Image::new_owned(icon_rgba.into_raw(), width, height)
            };
            
            #[cfg(not(target_os = "macos"))]
            let tray_icon = app.default_window_icon().unwrap().clone();

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)  // macOS: treat as template for dark/light mode
                .tooltip("Bingscape")
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
                            let _ = window.unminimize();
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
                        if let Ok(mut resolution) = app.state::<AppState>().resolution.lock() {
                            *resolution = settings.resolution;
                        }
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

            // Show and focus the window on initial startup
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }

            tauri::async_runtime::spawn(async move {
                loop {
                    if background_state.load(std::sync::atomic::Ordering::SeqCst) {
                        let apply_all = background_apply_all.load(std::sync::atomic::Ordering::SeqCst);
                        let sync_handle = handle.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            let state = sync_handle.state::<AppState>();
                            if let Err(err) = perform_sync(&sync_handle, &state, apply_all) {
                                if let Ok(mut guard) = state.last_status.lock() {
                                    guard.last_error = Some(err);
                                    guard.last_run = Some(Utc::now().to_rfc3339());
                                }
                            }
                        });
                    }
                    sleep(Duration::from_secs(60 * 60)).await;
                }
            });
            Ok(())
        })
        .manage(state)
        .invoke_handler(tauri::generate_handler![sync_wallpaper, set_auto_sync, get_status, get_settings, set_resolution, clear_cache])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
