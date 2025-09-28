#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::{AppHandle, CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent};
use serde::{Deserialize, Serialize};
use enigo::{Enigo, KeyboardControllable, MouseControllable};
use arboard::Clipboard;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
    pub timestamp: u64,
}

pub struct AppState {
    external_cursor_positions: Mutex<Vec<CursorPosition>>,
    is_focused: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            external_cursor_positions: Mutex::new(Vec::new()),
            is_focused: Mutex::new(false),
        }
    }
}

#[tauri::command]
async fn toggle_always_on_top(window: tauri::Window) -> Result<bool, String> {
    let is_always_on_top = window.is_always_on_top().map_err(|e| e.to_string())?;
    window.set_always_on_top(!is_always_on_top).map_err(|e| e.to_string())?;
    Ok(!is_always_on_top)
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
    let enigo = Enigo::new();
    let (x, y) = enigo.mouse_location();
    Ok(CursorPosition {
        x,
        y,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    })
}

#[tauri::command]
async fn check_app_focus(window: tauri::Window, state: State<'_, AppState>) -> Result<bool, String> {
    let is_focused = window.is_focused().map_err(|e| e.to_string())?;
    let mut app_focused = state.is_focused.lock().unwrap();
    *app_focused = is_focused;
    Ok(is_focused)
}

#[tauri::command]
async fn perform_injection_at_position(text: String, x: i32, y: i32) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_content = clipboard.get_text().unwrap_or_default();
    
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    
    let mut enigo = Enigo::new();
    enigo.mouse_move_to(x, y);
    thread::sleep(Duration::from_millis(50));
    enigo.mouse_click(enigo::MouseButton::Left);
    thread::sleep(Duration::from_millis(50));
    
    enigo.key_down(enigo::Key::Control);
    enigo.key_click(enigo::Key::Layout('v'));
    enigo.key_up(enigo::Key::Control);
    
    thread::sleep(Duration::from_millis(100));
    
    if !original_content.is_empty() {
        clipboard.set_text(&original_content).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn perform_injection(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_content = clipboard.get_text().unwrap_or_default();
    
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    
    let mut enigo = Enigo::new();
    enigo.key_down(enigo::Key::Control);
    enigo.key_click(enigo::Key::Layout('v'));
    enigo.key_up(enigo::Key::Control);
    
    thread::sleep(Duration::from_millis(100));
    
    if !original_content.is_empty() {
        clipboard.set_text(&original_content).map_err(|e| e.to_string())?;
    }
    
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

    tauri::Builder::default()
        .manage(AppState::default())
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                if window.is_visible().unwrap() {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "always_on_top" => {
                    let window = app.get_window("main").unwrap();
                    let is_always_on_top = window.is_always_on_top().unwrap();
                    window.set_always_on_top(!is_always_on_top).unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            toggle_always_on_top,
            set_window_position,
            get_window_position,
            minimize_to_tray,
            restore_from_tray,
            get_system_info,
            get_cursor_position,
            check_app_focus,
            perform_injection_at_position,
            perform_injection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}