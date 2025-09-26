// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    WindowBuilder, WindowEvent, WindowUrl,
};
use std::collections::HashMap;

// Imports cross-platform pour la détection curseur et injection
use enigo::*;
use mouse_position::get_mouse_position;

// Commande pour basculer always-on-top
#[tauri::command]
async fn toggle_always_on_top(window: tauri::Window) -> Result<bool, String> {
    match window.is_always_on_top() {
        Ok(current_state) => {
            let new_state = !current_state;
            match window.set_always_on_top(new_state) {
                Ok(_) => Ok(new_state),
                Err(e) => Err(format!("Erreur lors du changement always-on-top: {}", e)),
            }
        }
        Err(e) => Err(format!("Erreur lors de la lecture du statut: {}", e)),
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
    match get_mouse_position() {
        Ok((x, y)) => Ok((x, y)),
        Err(e) => Err(format!("Erreur position curseur: {}", e))
    }
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
    match tauri::api::clipboard::write_text(&text) {
        Ok(_) => {},
        Err(e) => return Err(format!("Erreur presse-papier: {}", e)),
    }
    
    // 3. Simuler raccourci de collage (Cmd+V sur macOS, Ctrl+V ailleurs)
    let mut enigo = Enigo::new();
    
    #[cfg(target_os = "macos")]
    {
        enigo.key_down(Key::Meta); // Touche Cmd sur macOS
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Meta);
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key_down(Key::Control); // Touche Ctrl sur Windows/Linux
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Control);
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
                        let current_state = window.is_always_on_top().unwrap_or(false);
                        let _ = window.set_always_on_top(!current_state);
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
            perform_injection
        ])
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de l'application");
}