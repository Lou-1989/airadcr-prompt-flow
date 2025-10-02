#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Mutex, Arc};
use tauri::{CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent};
use serde::{Deserialize, Serialize};
use enigo::{Enigo, Button, Key, Settings, Direction, Coordinate, Mouse, Keyboard};
use arboard::Clipboard;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use active_win_pos_rs::get_active_window;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
    pub timestamp: u64,
}

pub struct AppState {
    is_focused: Arc<Mutex<bool>>,
    always_on_top: Arc<Mutex<bool>>,
    clipboard_lock: Arc<Mutex<()>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_focused: Arc::new(Mutex::new(false)),
            always_on_top: Arc::new(Mutex::new(true)),
            clipboard_lock: Arc::new(Mutex::new(())),
        }
    }
}

#[tauri::command]
async fn toggle_always_on_top(window: tauri::Window, state: State<'_, AppState>) -> Result<bool, String> {
    let mut always_on_top = match state.always_on_top.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned in toggle_always_on_top, recovering...");
            poisoned.into_inner()
        }
    };
    
    let new_state = !*always_on_top;
    
    // Retry logic for window operations
    let mut retry_count = 0;
    while retry_count < 3 {
        match window.set_always_on_top(new_state) {
            Ok(_) => break,
            Err(_e) if retry_count < 2 => {
                retry_count += 1;
                thread::sleep(Duration::from_millis(50));
                continue;
            },
            Err(e) => return Err(format!("Failed to set always on top after retries: {}", e))
        }
    }
    
    *always_on_top = new_state;
    Ok(new_state)
}

#[tauri::command]
async fn set_always_on_top(window: tauri::Window, state: State<'_, AppState>, always_on_top: bool) -> Result<bool, String> {
    let mut current_state = match state.always_on_top.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned in set_always_on_top, recovering...");
            poisoned.into_inner()
        }
    };
    
    // Retry logic for window operations
    let mut retry_count = 0;
    while retry_count < 3 {
        match window.set_always_on_top(always_on_top) {
            Ok(_) => break,
            Err(_e) if retry_count < 2 => {
                retry_count += 1;
                thread::sleep(Duration::from_millis(50));
                continue;
            },
            Err(e) => return Err(format!("Failed to set always on top after retries: {}", e))
        }
    }
    
    *current_state = always_on_top;
    Ok(always_on_top)
}

#[tauri::command]
async fn set_window_position(window: tauri::Window, x: i32, y: i32) -> Result<(), String> {
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_window_position(window: tauri::Window) -> Result<(i32, i32), String> {
    let position = window.outer_position().map_err(|e| e.to_string())?;
    Ok((position.x, position.y))
}

#[tauri::command]
async fn minimize_to_tray(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn restore_from_tray(window: tauri::Window) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Serialize)]
pub struct SystemInfo {
    platform: String,
    arch: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowInfo {
    pub title: String,
    pub app_name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[tauri::command]
async fn get_system_info() -> Result<SystemInfo, String> {
    Ok(SystemInfo {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        version: "1.0.0".to_string(),
    })
}

#[tauri::command]
async fn get_cursor_position() -> Result<CursorPosition, String> {
    let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let (x, y) = enigo.location().map_err(|e| e.to_string())?;
    Ok(CursorPosition {
        x,
        y,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX as u128) as u64,
    })
}

#[tauri::command]
async fn check_app_focus(window: tauri::Window, state: State<'_, AppState>) -> Result<bool, String> {
    let is_focused = window.is_focused().map_err(|e| e.to_string())?;
    
    let mut app_focused = match state.is_focused.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned in check_app_focus, recovering...");
            poisoned.into_inner()
        }
    };
    
    *app_focused = is_focused;
    Ok(is_focused)
}

#[tauri::command]
async fn perform_injection_at_position(text: String, x: i32, y: i32, state: State<'_, AppState>) -> Result<(), String> {
    // Thread-safe clipboard operations
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_content = clipboard.get_text().unwrap_or_default();
    
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    
    // ⚡ OPTIMISATION: Délais réduits de 300ms → 60ms
    thread::sleep(Duration::from_millis(10));
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(10));
    enigo.button(Button::Left, Direction::Press).map_err(|e| e.to_string())?;
    enigo.button(Button::Left, Direction::Release).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(30));
    
    // Coller le texte à la position du curseur
    enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(10));
    
    if !original_content.is_empty() {
        clipboard.set_text(&original_content).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn perform_injection(text: String, state: State<'_, AppState>) -> Result<(), String> {
    // Thread-safe clipboard operations
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_content = clipboard.get_text().unwrap_or_default();
    
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(100));
    
    if !original_content.is_empty() {
        clipboard.set_text(&original_content).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
fn get_always_on_top_status(state: State<'_, AppState>) -> Result<bool, String> {
    let always_on_top = match state.always_on_top.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned in get_always_on_top_status, recovering...");
            poisoned.into_inner()
        }
    };
    
    Ok(*always_on_top)
}

#[tauri::command]
async fn set_ignore_cursor_events(window: tauri::Window, ignore: bool) -> Result<(), String> {
    window.set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())?;
    println!("🖱️  Click-through {}", if ignore { "activé" } else { "désactivé" });
    Ok(())
}

#[tauri::command]
async fn get_active_window_info() -> Result<WindowInfo, String> {
    match get_active_window() {
        Ok(active_window) => {
            Ok(WindowInfo {
                title: active_window.title,
                app_name: active_window.app_name,
                x: active_window.position.x as i32,
                y: active_window.position.y as i32,
                width: active_window.position.width as i32,
                height: active_window.position.height as i32,
            })
        },
        Err(_) => Err("Erreur récupération fenêtre active".to_string())
    }
}

#[tauri::command]
async fn perform_injection_at_position_direct(text: String, x: i32, y: i32, state: State<'_, AppState>) -> Result<(), String> {
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    println!("🎯 Injection Ctrl+V à position ({}, {}) - {} caractères", x, y, text.len());
    
    thread::sleep(Duration::from_millis(10));
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(10));
    enigo.button(Button::Left, Direction::Press).map_err(|e| e.to_string())?;
    enigo.button(Button::Left, Direction::Release).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(30));
    
    // ✅ SAUVEGARDE du clipboard original
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_clipboard = clipboard.get_text().unwrap_or_default();
    
    // Injection via Ctrl+V
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(10));
    
    enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    
    // ✅ RESTAURATION du clipboard original
    if !original_clipboard.is_empty() {
        clipboard.set_text(&original_clipboard).map_err(|e| e.to_string())?;
        println!("✅ Clipboard restauré ({} caractères)", original_clipboard.len());
    }
    
    println!("✅ Injection Ctrl+V réussie ({} caractères)", text.len());
    
    Ok(())
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quitter");
    let show = CustomMenuItem::new("show".to_string(), "Afficher");
    let hide = CustomMenuItem::new("hide".to_string(), "Masquer");
    let always_on_top = CustomMenuItem::new("always_on_top".to_string(), "Toujours au premier plan");

    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(always_on_top)
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    let app = tauri::Builder::default()
        .manage(AppState::default())
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                if let Some(window) = app.get_window("main") {
                    match window.is_visible() {
                        Ok(true) => { let _ = window.hide(); }
                        Ok(false) => { 
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        Err(e) => eprintln!("Error checking window visibility: {}", e),
                    }
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    // Graceful shutdown instead of brutal exit
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.hide();
                    }
                }
                "always_on_top" => {
                    if let Some(window) = app.get_window("main") {
                        let state = app.state::<AppState>();
                        let mut always_on_top = match state.always_on_top.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => {
                                eprintln!("Mutex poisoned in tray event, recovering...");
                                poisoned.into_inner()
                            }
                        };
                        let new_state = !*always_on_top;
                        if window.set_always_on_top(new_state).is_ok() {
                            *always_on_top = new_state;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                if let Err(e) = event.window().hide() {
                    eprintln!("Error hiding window on close: {}", e);
                }
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            toggle_always_on_top,
            set_always_on_top,
            set_window_position,
            get_window_position,
            minimize_to_tray,
            restore_from_tray,
            get_system_info,
            get_cursor_position,
            check_app_focus,
            get_always_on_top_status,
            perform_injection_at_position,
            perform_injection,
            set_ignore_cursor_events,
            perform_injection_at_position_direct,
            get_active_window_info
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Ready = event {
            if let Some(window) = app_handle.get_window("main") {
                // Activer UNIQUEMENT always-on-top au démarrage
                // Le click-through sera géré par useInteractionMode
                let _ = window.set_always_on_top(true);
                println!("✅ Always-on-top activé au démarrage");
            }
        }
    });
}