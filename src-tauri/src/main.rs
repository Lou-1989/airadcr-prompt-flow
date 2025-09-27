// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    WindowEvent,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use arboard::Clipboard;
use tokio;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
    pub timestamp: u64,
}

// Structure globale pour stocker l'état des positions
#[derive(Debug)]
pub struct AppState {
    pub external_positions: Arc<Mutex<Vec<CursorPosition>>>,
    pub app_has_focus: Arc<Mutex<bool>>,
}

// Imports cross-platform pour la détection curseur et injection
use enigo::{Enigo, Settings, Key, Direction, Keyboard};

// Commande pour basculer always-on-top
#[tauri::command]
async fn toggle_always_on_top(window: tauri::Window) -> Result<bool, String> {
    // Simuler un toggle simple - on met toujours à true pour le test
    // Dans une vraie implémentation, il faudrait stocker l'état
    match window.set_always_on_top(true) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Erreur lors du changement always-on-top: {}", e)),
    }
}

// Commande pour définir la position de la fenêtre
#[tauri::command]
async fn set_window_position(window: tauri::Window, x: i32, y: i32) -> Result<(), String> {
    match window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y })) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Erreur lors du positionnement: {}", e)),
    }
}

// Commande pour obtenir la position de la fenêtre
#[tauri::command]
async fn get_window_position(window: tauri::Window) -> Result<(i32, i32), String> {
    match window.outer_position() {
        Ok(position) => Ok((position.x, position.y)),
        Err(e) => Err(format!("Erreur lors de la lecture de position: {}", e)),
    }
}

// Commande pour minimiser vers le system tray
#[tauri::command]
async fn minimize_to_tray(window: tauri::Window) -> Result<(), String> {
    match window.hide() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Erreur lors de la minimisation: {}", e)),
    }
}

// Commande pour restaurer depuis le system tray
#[tauri::command]
async fn restore_from_tray(window: tauri::Window) -> Result<(), String> {
    match window.show() {
        Ok(_) => {
            match window.set_focus() {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Erreur lors de la focalisation: {}", e)),
            }
        }
        Err(e) => Err(format!("Erreur lors de la restauration: {}", e)),
    }
}

// Commande pour obtenir les informations système
#[tauri::command]
async fn get_system_info() -> Result<HashMap<String, String>, String> {
    let mut info = HashMap::new();
    
    info.insert("platform".to_string(), std::env::consts::OS.to_string());
    info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
    info.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    
    Ok(info)
}

// Commande pour obtenir la position du curseur (cross-platform)
#[tauri::command]
async fn get_cursor_position() -> Result<(i32, i32), String> {
    // Retourner une position par défaut pour le moment
    // Dans une vraie implémentation, utiliser une autre lib ou API système
    Ok((0, 0))
}

// Commande pour détecter si l'app a le focus
#[tauri::command]
async fn check_app_focus(window: tauri::Window) -> Result<bool, String> {
    match window.is_focused() {
        Ok(focused) => Ok(focused),
        Err(e) => Err(format!("Erreur vérification focus: {}", e)),
    }
}

// Commande pour effectuer l'injection à une position spécifique
#[tauri::command]
async fn perform_injection_at_position(text: String, x: i32, y: i32) -> Result<(i32, i32), String> {
    // 1. Copier le texte dans le presse-papier
    let mut clipboard = Clipboard::new().map_err(|e| format!("Erreur clipboard init: {}", e))?;
    clipboard.set_text(text).map_err(|e| format!("Erreur presse-papier: {}", e))?;
    
    // 2. Petit délai pour s'assurer que le focus est correct
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // 3. Simuler raccourci de collage (Cmd+V sur macOS, Ctrl+V ailleurs)
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Erreur enigo: {}", e))?;
    
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("Erreur key: {}", e))?;
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("Erreur key: {}", e))?;
    }
    
    Ok((x, y))
}

// Commande pour effectuer l'injection automatique (cross-platform)
#[tauri::command]
async fn perform_injection(text: String) -> Result<(i32, i32), String> {
    // 1. Récupérer la position du curseur
    let cursor_pos = match get_cursor_position().await {
        Ok(pos) => pos,
        Err(e) => return Err(format!("Erreur position curseur: {}", e)),
    };
    
    // 2. Copier le texte dans le presse-papier
    let mut clipboard = Clipboard::new().map_err(|e| format!("Erreur clipboard init: {}", e))?;
    clipboard.set_text(text).map_err(|e| format!("Erreur presse-papier: {}", e))?;
    
    // 3. Simuler raccourci de collage (Cmd+V sur macOS, Ctrl+V ailleurs)
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Erreur enigo: {}", e))?;
    
    #[cfg(target_os = "macos")]
    {
        enigo.key(Key::Meta, Direction::Press).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Meta, Direction::Release).map_err(|e| format!("Erreur key: {}", e))?;
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| format!("Erreur key: {}", e))?;
        enigo.key(Key::Control, Direction::Release).map_err(|e| format!("Erreur key: {}", e))?;
    }
    
    Ok(cursor_pos)
}

fn main() {
    // Configuration du menu System Tray
    let quit = CustomMenuItem::new("quit".to_string(), "Quitter");
    let show = CustomMenuItem::new("show".to_string(), "Afficher");
    let hide = CustomMenuItem::new("hide".to_string(), "Masquer");
    let always_on_top = CustomMenuItem::new("always_on_top".to_string(), "Always-on-top");
    
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(always_on_top)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let window = app.get_window("main").unwrap();
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    "hide" => {
                        let _ = window.hide();
                    }
                    "always_on_top" => {
                        // Simuler un toggle simple
                        let _ = window.set_always_on_top(true);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                // Au lieu de fermer, on masque la fenêtre
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
            perform_injection,
            perform_injection_at_position,
            check_app_focus
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de l'application");
}