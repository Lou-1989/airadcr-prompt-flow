#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Mutex, Arc, OnceLock};
use tauri::{CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent, GlobalShortcutManager, AppHandle};
use serde::{Deserialize, Serialize};
use enigo::{Enigo, Button, Key, Settings, Direction, Coordinate, Mouse, Keyboard};
use arboard::Clipboard;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use active_win_pos_rs::get_active_window;
use log::{info, warn, error, debug};

/// Retourne la touche modificateur pour copier/coller selon l'OS
/// Windows/Linux: Ctrl, macOS: Cmd (Meta)
fn clipboard_modifier() -> Key {
    #[cfg(target_os = "macos")]
    { Key::Meta }
    #[cfg(not(target_os = "macos"))]
    { Key::Control }
}
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
extern crate chrono;

// 🌐 Global AppHandle pour communication HTTP → Tauri
pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

// 🌐 Modules serveur HTTP et base de données
mod http_server;
mod database;
mod teo_client;
mod config;
mod speechmike;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};

// 🆕 Fonction pour activer DPI Per-Monitor V2 (multi-écrans + multi-DPI)
#[cfg(target_os = "windows")]
fn enable_dpi_awareness() {
    use winapi::um::winuser::SetProcessDpiAwarenessContext;
    use winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
    
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == 0 {
            warn!("Échec activation DPI Per-Monitor V2, fallback mode par défaut");
        } else {
            info!("DPI Per-Monitor V2 activé (coordonnées physiques multi-écrans)");
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CursorPosition {
    pub x: i32,
    pub y: i32,
    pub timestamp: u64,
}

// ✅ AppState simplifié (dictation_state supprimé)
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
            warn!("Mutex poisoned in toggle_always_on_top, recovering...");
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
            warn!("Mutex poisoned in set_always_on_top, recovering...");
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
        // 🆕 Version dynamique depuis Cargo.toml (Phase 1)
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
async fn get_cursor_position() -> Result<CursorPosition, String> {
    // Retry logic pour gérer les erreurs temporaires multi-écrans
    let mut retry_count = 0;
    let max_retries = 5; // 🆕 Augmenté de 3 → 5 pour multi-écrans
    
    while retry_count < max_retries {
        let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        match enigo.location() {
            Ok((x, y)) => {
                return Ok(CursorPosition {
                    x,
                    y,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                        .min(u64::MAX as u128) as u64,
                });
            },
            Err(_e) if retry_count < max_retries - 1 => {
                retry_count += 1;
                thread::sleep(Duration::from_millis(150)); // 🆕 Augmenté de 100ms → 150ms
                continue;
            },
            Err(e) => return Err(format!("Failed to get cursor position after {} retries: {}", max_retries, e))
        }
    }
    
    Err("Failed to get cursor position".to_string())
}

#[tauri::command]
async fn check_app_focus(window: tauri::Window, state: State<'_, AppState>) -> Result<bool, String> {
    let is_focused = window.is_focused().map_err(|e| e.to_string())?;
    
    let mut app_focused = match state.is_focused.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Mutex poisoned in check_app_focus, recovering...");
            poisoned.into_inner()
        }
    };
    
    *app_focused = is_focused;
    Ok(is_focused)
}

#[tauri::command]
async fn perform_injection_at_position(text: String, html: Option<String>, x: i32, y: i32, state: State<'_, AppState>) -> Result<(), String> {
    // Thread-safe clipboard operations
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_content = clipboard.get_text().unwrap_or_default();
    
    // 🆕 Rich Text: utiliser set_html si du HTML est fourni, sinon set_text
    if let Some(ref html_content) = html {
        clipboard.set_html(html_content, Some(&text)).map_err(|e| e.to_string())?;
        debug!("[Injection] Clipboard HTML: {} chars HTML + {} chars texte fallback", html_content.len(), text.len());
    } else {
        clipboard.set_text(&text).map_err(|e| e.to_string())?;
    }
    
    // ⚡ OPTIMISATION: Délais réduits de 300ms → 60ms
    thread::sleep(Duration::from_millis(10));
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(10));
    enigo.button(Button::Left, Direction::Press).map_err(|e| e.to_string())?;
    enigo.button(Button::Left, Direction::Release).map_err(|e| e.to_string())?;
    thread::sleep(Duration::from_millis(30));
    
    // Coller le texte à la position du curseur (Cmd+V sur macOS, Ctrl+V sur Windows/Linux)
    let paste_key = clipboard_modifier();
    enigo.key(paste_key, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(paste_key, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(10));
    
    if !original_content.is_empty() {
        clipboard.set_text(&original_content).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn perform_injection(text: String, html: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    // Thread-safe clipboard operations
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_content = clipboard.get_text().unwrap_or_default();
    
    // 🆕 Rich Text: utiliser set_html si du HTML est fourni, sinon set_text
    if let Some(ref html_content) = html {
        clipboard.set_html(html_content, Some(&text)).map_err(|e| e.to_string())?;
        debug!("[Injection] Clipboard HTML: {} chars HTML + {} chars texte fallback", html_content.len(), text.len());
    } else {
        clipboard.set_text(&text).map_err(|e| e.to_string())?;
    }
    
    thread::sleep(Duration::from_millis(50));
    
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let paste_key = clipboard_modifier();
    enigo.key(paste_key, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(paste_key, Direction::Release).map_err(|e| e.to_string())?;
    
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
            warn!("Mutex poisoned in get_always_on_top_status, recovering...");
            poisoned.into_inner()
        }
    };
    
    Ok(*always_on_top)
}

// 🆕 DÉTECTION DE SÉLECTION DE TEXTE
#[tauri::command]
async fn has_text_selection(state: State<'_, AppState>) -> Result<bool, String> {
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    // Sauvegarder le clipboard actuel
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original = clipboard.get_text().unwrap_or_default();
    
    // Simuler Ctrl+C (Windows/Linux) ou Cmd+C (macOS) pour copier la sélection
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let copy_key = clipboard_modifier();
    enigo.key(copy_key, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(copy_key, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    
    // Vérifier si le clipboard a changé
    let after_copy = clipboard.get_text().unwrap_or_default();
    let has_selection = !after_copy.is_empty() && after_copy != original;
    
    // Restaurer le clipboard original
    if original != after_copy {
        clipboard.set_text(&original).map_err(|e| e.to_string())?;
    }
    
    debug!("Détection sélection: {}", if has_selection { "OUI" } else { "NON" });
    Ok(has_selection)
}

#[tauri::command]
async fn set_ignore_cursor_events(window: tauri::Window, ignore: bool) -> Result<(), String> {
    window.set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())?;
    debug!("Click-through {}", if ignore { "activé" } else { "désactivé" });
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

#[cfg(target_os = "windows")]
use winapi::um::winuser::SetCursorPos;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{WindowFromPoint, GetAncestor, SetForegroundWindow, GA_ROOT, GetForegroundWindow, GetWindowRect, IsIconic, ShowWindow, GetWindowPlacement, SetWindowPlacement, SW_RESTORE, SW_SHOWNORMAL};
#[cfg(target_os = "windows")]
use winapi::shared::windef::{POINT, RECT};
#[cfg(target_os = "windows")]
use winapi::um::winuser::WINDOWPLACEMENT;

// 🆕 INJECTION WINDOWS ROBUSTE avec Win32 API pour multi-écrans
#[tauri::command]
async fn perform_injection_at_position_direct(x: i32, y: i32, text: String, html: Option<String>, state: State<'_, AppState>) -> Result<(), String> {
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    info!("[Injection Rust] Début - Position: ({}, {}) - Texte: {} chars", x, y, text.len());
    let start_time = std::time::Instant::now();
    
    debug!("[Multi-écrans] Injection à ({}, {}) - {} caractères", x, y, text.len());
    
    // 🆕 CLAMPER les coordonnées dans les bornes du bureau virtuel
    #[cfg(target_os = "windows")]
    let (clamped_x, clamped_y) = unsafe {
        let vd_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vd_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vd_width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let vd_height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        
        let cx = x.max(vd_x).min(vd_x + vd_width - 1);
        let cy = y.max(vd_y).min(vd_y + vd_height - 1);
        
        if cx != x || cy != y {
            debug!("[Multi-écrans] Coordonnées clampées: ({}, {}) → ({}, {}) [Bureau: ({}, {}) {}x{}]", 
                x, y, cx, cy, vd_x, vd_y, vd_width, vd_height);
        }
        
        (cx, cy)
    };
    
    #[cfg(not(target_os = "windows"))]
    let (clamped_x, clamped_y) = (x, y);
    
    if clamped_x < 0 || clamped_y < 0 {
        debug!("[Multi-écrans] Coordonnées négatives détectées (écran secondaire gauche/haut)");
    }
    
    // LOG: Fenêtre active AVANT clic
    match get_active_window() {
        Ok(win) => debug!("[Avant clic] Fenêtre active: {} ({})", win.app_name, win.title),
        Err(_) => debug!("[Avant clic] Impossible de récupérer la fenêtre active")
    }
    
    thread::sleep(Duration::from_millis(10));
    
    // 🆕 WINDOWS: Utiliser SetCursorPos (Win32) pour multi-écrans robuste
    #[cfg(target_os = "windows")]
    {
        unsafe {
            if SetCursorPos(clamped_x, clamped_y) == 0 {
                return Err("Échec SetCursorPos (Win32)".to_string());
            }
        }
        debug!("[Win32] SetCursorPos({}, {}) réussi", clamped_x, clamped_y);
        
        // 🆕 FORCER LE FOCUS sur la fenêtre sous le curseur (multi-écrans)
        unsafe {
            let point = POINT { x: clamped_x, y: clamped_y };
            let hwnd = WindowFromPoint(point);
            
            if !hwnd.is_null() {
                let root_hwnd = GetAncestor(hwnd, GA_ROOT);
                if !root_hwnd.is_null() {
                    // Vérifier si la fenêtre est minimisée
                    if IsIconic(root_hwnd) != 0 {
                        debug!("[Restauration] Fenêtre minimisée détectée, restauration...");
                        ShowWindow(root_hwnd, SW_RESTORE);
                        thread::sleep(Duration::from_millis(200));
                        debug!("[Restauration] Fenêtre restaurée depuis l'état minimisé");
                    }
                    
                    // Vérifier si la fenêtre est hors écran
                    let mut placement: WINDOWPLACEMENT = std::mem::zeroed();
                    placement.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;
                    
                    if GetWindowPlacement(root_hwnd, &mut placement) != 0 {
                        let rect = placement.rcNormalPosition;
                        
                        // Obtenir les bornes du bureau virtuel
                        let vd_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
                        let vd_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
                        let vd_width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
                        let vd_height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
                        let vd_right = vd_x + vd_width;
                        let vd_bottom = vd_y + vd_height;
                        
                        // Vérifier si la fenêtre est complètement hors des bornes
                        let is_offscreen = rect.right < vd_x || rect.left > vd_right ||
                                         rect.bottom < vd_y || rect.top > vd_bottom;
                        
                        if is_offscreen {
                            debug!("[Restauration] Fenêtre hors écran détectée: ({}, {}) - ({}, {})", 
                                rect.left, rect.top, rect.right, rect.bottom);
                            debug!("   Bureau virtuel: ({}, {}) - ({}, {})", vd_x, vd_y, vd_right, vd_bottom);
                            
                            // Repositionner la fenêtre au centre du bureau principal
                            let new_x = vd_x + (vd_width / 4);
                            let new_y = vd_y + (vd_height / 4);
                            let win_width = rect.right - rect.left;
                            let win_height = rect.bottom - rect.top;
                            
                            placement.rcNormalPosition.left = new_x;
                            placement.rcNormalPosition.top = new_y;
                            placement.rcNormalPosition.right = new_x + win_width;
                            placement.rcNormalPosition.bottom = new_y + win_height;
                            placement.showCmd = SW_SHOWNORMAL as u32;
                            
                            if SetWindowPlacement(root_hwnd, &placement) != 0 {
                                debug!("[Restauration] Fenêtre repositionnée à ({}, {})", new_x, new_y);
                                thread::sleep(Duration::from_millis(150));
                            } else {
                                warn!("[Restauration] SetWindowPlacement a échoué");
                            }
                        }
                    }
                    
                    // Focus final avec vérification
                    if SetForegroundWindow(root_hwnd) != 0 {
                        debug!("[Multi-écrans] Focus forcé sur fenêtre à ({}, {})", clamped_x, clamped_y);
                    } else {
                        warn!("[Multi-écrans] SetForegroundWindow a échoué");
                    }
                }
            }
        }
        
        thread::sleep(Duration::from_millis(120)); // Stabiliser focus
    }
    
    // FALLBACK: Autres OS utilisent Enigo
    #[cfg(not(target_os = "windows"))]
    {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        enigo.move_mouse(clamped_x, clamped_y, Coordinate::Abs).map_err(|e| e.to_string())?;
    }
    
    debug!("[Injection] Position finale: ({}, {}) - SANS CLIC (préserve sélection)", clamped_x, clamped_y);
    thread::sleep(Duration::from_millis(10));
    
    // Créer enigo pour Ctrl+V
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    
    // LOG: Fenêtre active AVANT Ctrl+V
    match get_active_window() {
        Ok(win) => debug!("[Avant Ctrl+V] Fenêtre active: {} ({})", win.app_name, win.title),
        Err(_) => debug!("[Avant Ctrl+V] Impossible de récupérer la fenêtre active")
    }
    
    // SAUVEGARDE du clipboard original
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_clipboard = clipboard.get_text().unwrap_or_default();
    debug!("Clipboard sauvegardé : {} caractères", original_clipboard.len());
    
    // Injection via Ctrl+V (remplace la sélection manuelle utilisateur si présente)
    // 🆕 Rich Text: utiliser set_html si du HTML est fourni, sinon set_text
    if let Some(ref html_content) = html {
        clipboard.set_html(html_content, Some(&text)).map_err(|e| e.to_string())?;
        debug!("HTML copié dans clipboard : {} chars HTML + {} chars texte fallback", html_content.len(), text.len());
    } else {
        clipboard.set_text(&text).map_err(|e| e.to_string())?;
        debug!("Texte copié dans clipboard : {} caractères", text.len());
    }
    thread::sleep(Duration::from_millis(10));
    
    debug!("Envoi Paste (va remplacer la sélection si présente, sinon coller)");
    let paste_key = clipboard_modifier();
    enigo.key(paste_key, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(paste_key, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    debug!("Injection Ctrl+V terminée");
    
    // RESTAURATION du clipboard original
    if !original_clipboard.is_empty() {
        clipboard.set_text(&original_clipboard).map_err(|e| e.to_string())?;
        debug!("Clipboard restauré ({} caractères)", original_clipboard.len());
    }
    
    info!("Injection Ctrl+V réussie ({} caractères) en {}ms", text.len(), start_time.elapsed().as_millis());
    
    Ok(())
}

// 🆕 Commande: Récupérer la fenêtre sous le curseur (même si AirADCR a le focus)
#[tauri::command]
async fn get_window_at_point(x: i32, y: i32) -> Result<WindowInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use winapi::shared::windef::POINT;
        
        unsafe {
            let point = POINT { x, y };
            let hwnd = WindowFromPoint(point);
            
            if hwnd.is_null() {
                return Err("Aucune fenêtre trouvée à cette position".to_string());
            }
            
            let root_hwnd = GetAncestor(hwnd, GA_ROOT);
            if root_hwnd.is_null() {
                return Err("Impossible de récupérer la fenêtre racine".to_string());
            }
            
            // Utiliser get_active_window pour récupérer les infos
            match get_active_window() {
                Ok(win) => Ok(WindowInfo {
                    title: win.title,
                    app_name: win.app_name,
                    x: win.position.x as i32,
                    y: win.position.y as i32,
                    width: win.position.width as i32,
                    height: win.position.height as i32,
                }),
                Err(_) => {
                    // Fallback: retourner info minimale
                    let mut rect: RECT = std::mem::zeroed();
                    if GetWindowRect(root_hwnd, &mut rect) != 0 {
                        Ok(WindowInfo {
                            title: "Unknown".to_string(),
                            app_name: "Unknown".to_string(),
                            x: rect.left,
                            y: rect.top,
                            width: rect.right - rect.left,
                            height: rect.bottom - rect.top,
                        })
                    } else {
                        Err("Impossible de récupérer les infos de la fenêtre".to_string())
                    }
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback macOS/Linux: utiliser active_win_pos_rs
        match get_active_window() {
            Ok(win) => Ok(WindowInfo {
                title: win.title,
                app_name: win.app_name,
                x: win.position.x as i32,
                y: win.position.y as i32,
                width: win.position.width as i32,
                height: win.position.height as i32,
            }),
            Err(_) => Ok(WindowInfo {
                title: "Unknown".to_string(),
                app_name: "Unknown".to_string(),
                x: 0, y: 0, width: 1920, height: 1080,
            })
        }
    }
}

// 🎤 Helper: Convertir nom de touche → keyCode JavaScript
fn get_key_code(key_name: &str) -> u32 {
    match key_name {
        "F10" => 121,
        "F11" => 122,
        "F12" => 123,
        _ => 0,
    }
}

// 🆕 Commande: Obtenir les infos du bureau virtuel Windows (multi-écrans)
#[derive(Serialize)]
pub struct VirtualDesktopInfo {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[tauri::command]
async fn get_virtual_desktop_info() -> Result<VirtualDesktopInfo, String> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
            
            debug!("[Multi-écrans] Bureau virtuel: ({}, {}) {}x{} | DPI Per-Monitor V2: ACTIVÉ", x, y, width, height);
            
            Ok(VirtualDesktopInfo { x, y, width, height })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback macOS/Linux: valeurs génériques du bureau principal
        Ok(VirtualDesktopInfo { x: 0, y: 0, width: 1920, height: 1080 })
    }
}

// 🆕 Commande: Obtenir les dimensions physiques de la fenêtre au premier plan (DPI-safe)
#[derive(Serialize)]
pub struct PhysicalRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub width: i32,
    pub height: i32,
}

#[tauri::command]
async fn get_physical_window_rect() -> Result<PhysicalRect, String> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return Err("Aucune fenêtre au premier plan".to_string());
            }
            
            let mut rect: RECT = std::mem::zeroed();
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return Err("GetWindowRect a échoué".to_string());
            }
            
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;
            
            debug!("[DPI] GetWindowRect physique: ({}, {}) {}x{}", 
                rect.left, rect.top, width, height);
            
            Ok(PhysicalRect {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
                width,
                height,
            })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback macOS/Linux: utiliser active_win_pos_rs
        match get_active_window() {
            Ok(win) => Ok(PhysicalRect {
                left: win.position.x as i32,
                top: win.position.y as i32,
                right: (win.position.x + win.position.width) as i32,
                bottom: (win.position.y + win.position.height) as i32,
                width: win.position.width as i32,
                height: win.position.height as i32,
            }),
            Err(_) => Ok(PhysicalRect {
                left: 0, top: 0, right: 1920, bottom: 1080, width: 1920, height: 1080,
            })
        }
    }
}

// 🆕 Commande: Obtenir les dimensions CLIENT RECT d'une fenêtre à une position (multi-écrans + DPI-safe)
#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetClientRect, ClientToScreen};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientRectInfo {
    pub app_name: String,
    pub title: String,
    pub window_left: i32,
    pub window_top: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub client_left: i32,
    pub client_top: i32,
    pub client_width: i32,
    pub client_height: i32,
    pub hwnd: usize, // 🆕 Handle de fenêtre Windows pour identification unique
}

#[tauri::command]
async fn get_window_client_rect_at_point(x: i32, y: i32) -> Result<ClientRectInfo, String> {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            // 1️⃣ Trouver la fenêtre sous le point
            let point = POINT { x, y };
            let hwnd = WindowFromPoint(point);
            
            if hwnd.is_null() {
                return Err("Aucune fenêtre trouvée à cette position".to_string());
            }
            
            // 2️⃣ Obtenir la fenêtre racine
            let root_hwnd = GetAncestor(hwnd, GA_ROOT);
            if root_hwnd.is_null() {
                return Err("Impossible d'obtenir la fenêtre racine".to_string());
            }
            
            // 3️⃣ GetWindowRect (dimensions externes avec bordures/titre)
            let mut window_rect: RECT = std::mem::zeroed();
            if GetWindowRect(root_hwnd, &mut window_rect) == 0 {
                return Err("GetWindowRect a échoué".to_string());
            }
            
            // 4️⃣ GetClientRect (dimensions internes SANS bordures/titre)
            let mut client_rect: RECT = std::mem::zeroed();
            if GetClientRect(root_hwnd, &mut client_rect) == 0 {
                return Err("GetClientRect a échoué".to_string());
            }
            
            // 5️⃣ Convertir le coin client (0,0) en coordonnées écran
            let mut client_origin = POINT { x: 0, y: 0 };
            if ClientToScreen(root_hwnd, &mut client_origin) == 0 {
                return Err("ClientToScreen a échoué".to_string());
            }
            
            // 6️⃣ Récupérer les infos de la fenêtre via active-win-pos-rs
            let (app_name, title) = match get_active_window() {
                Ok(win) => (win.app_name, win.title),
                Err(_) => ("Unknown".to_string(), "Unknown".to_string())
            };
            
            let window_width = window_rect.right - window_rect.left;
            let window_height = window_rect.bottom - window_rect.top;
            let client_width = client_rect.right - client_rect.left;
            let client_height = client_rect.bottom - client_rect.top;
            
            debug!("[ClientRect] Fenêtre: {} ({}, {}) {}x{} [HWND: {:?}]", 
                app_name, window_rect.left, window_rect.top, window_width, window_height, root_hwnd);
            debug!("[ClientRect] Zone client: ({}, {}) {}x{}", 
                client_origin.x, client_origin.y, client_width, client_height);
            
            Ok(ClientRectInfo {
                app_name,
                title,
                window_left: window_rect.left,
                window_top: window_rect.top,
                window_width,
                window_height,
                client_left: client_origin.x,
                client_top: client_origin.y,
                client_width,
                client_height,
                hwnd: root_hwnd as usize,
            })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // Fallback macOS/Linux: utiliser active_win_pos_rs
        match get_active_window() {
            Ok(win) => Ok(ClientRectInfo {
                app_name: win.app_name,
                title: win.title,
                window_left: win.position.x as i32,
                window_top: win.position.y as i32,
                window_width: win.position.width as i32,
                window_height: win.position.height as i32,
                client_left: win.position.x as i32,
                client_top: win.position.y as i32,
                client_width: win.position.width as i32,
                client_height: win.position.height as i32,
                hwnd: 0,
            }),
            Err(_) => Ok(ClientRectInfo {
                app_name: "Unknown".to_string(), title: "Unknown".to_string(),
                window_left: 0, window_top: 0, window_width: 1920, window_height: 1080,
                client_left: 0, client_top: 0, client_width: 1920, client_height: 1080,
                hwnd: 0,
            })
        }
    }
}

// ✅ handle_recording_notification supprimé (DictationState supprimé)


// 🎤 Commande: Simuler une touche dans l'iframe airadcr.com (legacy pour compatibilité)
#[tauri::command]
async fn simulate_key_in_iframe(window: tauri::Window, key: String) -> Result<(), String> {
    // 🔒 SÉCURITÉ: Valider la touche contre une liste blanche AVANT insertion JS
    let valid_keys = ["F10", "F11", "F12"];
    if !valid_keys.contains(&key.as_str()) {
        return Err(format!("Touche non autorisée: {}. Touches valides: {:?}", key, valid_keys));
    }
    
    let key_code = get_key_code(&key);
    
    debug!("[SpeechMike] Injection touche {} (code: {}) dans iframe", key, key_code);
    
    // Injection JavaScript: Dispatcher un KeyboardEvent dans l'iframe
    let js_code = format!(
        r#"
        try {{
            const iframe = document.querySelector('iframe');
            if (iframe && iframe.contentWindow) {{
                const event = new KeyboardEvent('keydown', {{
                    key: '{}',
                    code: '{}',
                    keyCode: {},
                    which: {},
                    bubbles: true,
                    cancelable: true
                }});
                iframe.contentWindow.document.dispatchEvent(event);
                console.log('✅ [SpeechMike] Événement {} injecté dans iframe');
            }} else {{
                console.error('❌ [SpeechMike] Iframe non trouvée');
            }}
        }} catch (error) {{
            console.error('❌ [SpeechMike] Erreur injection:', error);
        }}
        "#,
        key, key, key_code, key_code, key
    );
    
    window.eval(&js_code).map_err(|e| e.to_string())?;
    
    Ok(())
}

// 🆕 COMMANDE DE LOGGING PERSISTANT
#[tauri::command]
async fn write_log(message: String, level: String) -> Result<(), String> {
    // Obtenir le chemin du dossier de logs (cross-platform)
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("AIRADCR")
        .join("logs");
    
    // Créer le dossier si nécessaire
    if let Err(e) = create_dir_all(&log_dir) {
        return Err(format!("Impossible de créer le dossier de logs: {}", e));
    }
    
    // Créer/ouvrir le fichier de log avec append
    let log_path = log_dir.join("app.log");
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path) {
            Ok(f) => f,
            Err(e) => return Err(format!("Impossible d'ouvrir le fichier de log: {}", e)),
        };
    
    // Formater le timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    
    // Écrire le log
    let log_entry = format!("[{}] [{}] {}\n", 
        datetime.format("%Y-%m-%d %H:%M:%S"),
        level.to_uppercase(),
        message
    );
    
    if let Err(e) = file.write_all(log_entry.as_bytes()) {
        return Err(format!("Impossible d'écrire dans le fichier de log: {}", e));
    }
    
    Ok(())
}

// 🆕 COMMANDE POUR OBTENIR LE CHEMIN DES LOGS
#[tauri::command]
async fn get_log_path() -> Result<String, String> {
    let log_path = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("AIRADCR")
        .join("logs")
        .join("app.log");
    Ok(log_path.to_string_lossy().to_string())
}

// 🆕 COMMANDE POUR OUVRIR LE DOSSIER DES LOGS
#[tauri::command]
async fn open_log_folder() -> Result<(), String> {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("AIRADCR")
        .join("logs");
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(log_dir.to_string_lossy().as_ref())
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir le dossier: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(log_dir.to_string_lossy().as_ref())
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir le dossier: {}", e))?;
    }
    
    Ok(())
}

// ============================================================================
// 🎤 COMMANDES SPEECHMIKE NATIF (HID USB DIRECT)
// ============================================================================

#[tauri::command]
fn speechmike_get_status(state: State<'_, std::sync::Arc<speechmike::SpeechMikeState>>) -> Result<speechmike::SpeechMikeStatus, String> {
    let status = state.status.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(status.clone())
}

#[tauri::command]
fn speechmike_list_devices() -> Result<Vec<serde_json::Value>, String> {
    let api = hidapi::HidApi::new().map_err(|e| format!("HidApi error: {}", e))?;
    let mut devices = Vec::new();
    
    for device_info in api.device_list() {
        let vid = device_info.vendor_id();
        let pid = device_info.product_id();
        
        if let Some(filter) = speechmike::devices::is_supported_device(vid, pid) {
            devices.push(serde_json::json!({
                "vendor_id": format!("0x{:04x}", vid),
                "product_id": format!("0x{:04x}", pid),
                "description": filter.description,
                "manufacturer": device_info.manufacturer_string().unwrap_or("Unknown"),
                "product": device_info.product_string().unwrap_or("Unknown"),
                "serial": device_info.serial_number().unwrap_or(""),
            }));
        }
    }
    
    Ok(devices)
}

#[tauri::command]
fn speechmike_set_led(led_state: String, state: State<'_, std::sync::Arc<speechmike::SpeechMikeState>>) -> Result<(), String> {
    let status = state.status.lock().map_err(|e| format!("Lock error: {}", e))?;
    if !status.connected {
        return Err("Aucun SpeechMike connecté".to_string());
    }
    drop(status); // Release lock before acquiring another
    
    let simple_state = match led_state.as_str() {
        "recording" => speechmike::devices::SimpleLedState::RecordOverwrite,         // Rouge fixe
        "pause"     => speechmike::devices::SimpleLedState::RecordStandbyOverwrite,  // Rouge clignotant
        "idle"      => speechmike::devices::SimpleLedState::RecordInsert,            // Vert fixe
        "off"       => speechmike::devices::SimpleLedState::Off,
        _ => return Err(format!("État LED inconnu: {}", led_state)),
    };
    
    // Send LED command via channel to polling thread (no double-open HID)
    let led_tx = state.led_tx.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    match led_tx.as_ref() {
        Some(tx) => {
            tx.send(simple_state).map_err(|e| format!("LED channel send error: {}", e))?;
            info!("[SpeechMike] LED → {} (via channel)", led_state);
            Ok(())
        }
        None => Err("LED channel not available (device not connected)".to_string()),
    }
}

// ============================================================================
// COMMANDES POUR LE DEBUG PANEL - ONGLET BASE DE DONNÉES
// ============================================================================

/// Récupère tous les rapports en attente (pour Debug Panel)
#[tauri::command]
async fn get_all_pending_reports() -> Result<Vec<database::queries::PendingReportSummary>, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    db.list_all_pending_reports()
        .map_err(|e| format!("Erreur lecture rapports: {}", e))
}

/// Récupère la liste des clés API (sans données sensibles)
#[tauri::command]
async fn get_api_keys_list() -> Result<Vec<database::queries::ApiKeySummary>, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    db.list_api_keys_summary()
        .map_err(|e| format!("Erreur lecture clés API: {}", e))
}

/// Récupère les statistiques de la base de données
#[tauri::command]
async fn get_database_stats() -> Result<database::queries::DatabaseStats, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    db.get_database_stats()
        .map_err(|e| format!("Erreur lecture stats: {}", e))
}

/// Nettoie les rapports expirés (pour Debug Panel)
#[tauri::command]
async fn cleanup_expired_reports_cmd() -> Result<usize, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    db.cleanup_expired_reports()
        .map_err(|e| format!("Erreur cleanup: {}", e))
}

/// Supprime un rapport par son technical_id (pour Debug Panel)
#[tauri::command]
async fn delete_pending_report_cmd(technical_id: String) -> Result<bool, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    db.delete_pending_report(&technical_id)
        .map_err(|e| format!("Erreur suppression: {}", e))
}

/// Structure pour retourner une nouvelle clé API créée
#[derive(Serialize)]
struct NewApiKeyResult {
    key_prefix: String,
    full_key: String,
    name: String,
}

/// Génère et crée une nouvelle clé API (pour Debug Panel)
#[tauri::command]
async fn create_api_key_cmd(name: String) -> Result<NewApiKeyResult, String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    // Générer un ID unique
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let id = format!("key_{}", timestamp);
    
    // Générer une clé aléatoire (32 caractères hex)
    let random_bytes: Vec<u8> = (0..16).map(|_| rand::random::<u8>()).collect();
    let key_hex: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    
    // Préfixe de la clé (premiers 8 caractères)
    let key_prefix = format!("airadcr_{}", &key_hex[..8]);
    
    // Clé complète
    let full_key = format!("{}_{}", key_prefix, &key_hex[8..]);
    
    // 🛡️ SÉCURITÉ: Hash SHA-256 (aligné avec le serveur HTTP)
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(full_key.as_bytes());
    let key_hash = hex::encode(hasher.finalize());
    
    // Sauvegarder dans la DB
    db.add_api_key(&id, &key_prefix, &key_hash, &name)
        .map_err(|e| format!("Erreur création clé: {}", e))?;
    
    info!("[API Key] Nouvelle clé créée: {} ({})", name, key_prefix);
    
    Ok(NewApiKeyResult {
        key_prefix,
        full_key,
        name,
    })
}

/// Révoque une clé API (pour Debug Panel)
#[tauri::command]
async fn revoke_api_key_cmd(key_prefix: String) -> Result<bool, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    db.revoke_api_key(&key_prefix)
        .map_err(|e| format!("Erreur révocation: {}", e))
}

// =========================================================================
// Commandes Access Logs (AUDIT)
// =========================================================================

/// Structure pour les logs d'accès (sérialisation)
#[derive(Serialize)]
struct AccessLogSummary {
    id: i64,
    timestamp: String,
    ip_address: String,
    method: String,
    endpoint: String,
    status_code: i32,
    result: String,
    duration_ms: i64,
}

/// Structure pour les statistiques des logs
#[derive(Serialize)]
struct AccessLogsStats {
    total_requests: i64,
    success_count: i64,
    unauthorized_count: i64,
    error_count: i64,
    not_found_count: i64,
    requests_by_endpoint: Vec<(String, i64)>,
    avg_response_time_ms: f64,
    last_24h_requests: i64,
}

/// Liste les logs d'accès API récents (pour Debug Panel)
#[tauri::command]
async fn get_access_logs(limit: Option<i64>, offset: Option<i64>) -> Result<Vec<AccessLogSummary>, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    let logs = db.list_access_logs(limit.unwrap_or(100), offset.unwrap_or(0))
        .map_err(|e| format!("Erreur lecture logs: {}", e))?;
    
    // Convertir en structure locale
    let result: Vec<AccessLogSummary> = logs.into_iter().map(|log| AccessLogSummary {
        id: log.id,
        timestamp: log.timestamp,
        ip_address: log.ip_address,
        method: log.method,
        endpoint: log.endpoint,
        status_code: log.status_code,
        result: log.result,
        duration_ms: log.duration_ms,
    }).collect();
    
    Ok(result)
}

/// Récupère les statistiques des logs d'accès (pour Debug Panel)
#[tauri::command]
async fn get_access_logs_stats() -> Result<AccessLogsStats, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    let stats = db.get_access_logs_stats()
        .map_err(|e| format!("Erreur stats logs: {}", e))?;
    
    Ok(AccessLogsStats {
        total_requests: stats.total_requests,
        success_count: stats.success_count,
        unauthorized_count: stats.unauthorized_count,
        error_count: stats.error_count,
        not_found_count: stats.not_found_count,
        requests_by_endpoint: stats.requests_by_endpoint,
        avg_response_time_ms: stats.avg_response_time_ms,
        last_24h_requests: stats.last_24h_requests,
    })
}

/// Nettoie les vieux logs d'accès (pour Debug Panel)
#[tauri::command]
async fn cleanup_access_logs(days: Option<i64>) -> Result<i64, String> {
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = database::Database::new(app_data_dir)
        .map_err(|e| format!("Erreur ouverture DB: {}", e))?;
    
    let count = db.cleanup_old_access_logs(days.unwrap_or(30))
        .map_err(|e| format!("Erreur cleanup logs: {}", e))?;
    
    info!("[Access Logs] {} log(s) supprimé(s)", count);
    
    Ok(count as i64)
}

// ============================================================================
// COMMANDES TAURI - TÉO HUB CLIENT
// ============================================================================

/// Vérifie la disponibilité du serveur TÉO Hub
#[tauri::command]
async fn teo_check_health() -> Result<teo_client::models::TeoHealthResponse, String> {
    teo_client::check_health().await.map_err(|e| e.to_string())
}

/// Récupère un rapport IA depuis TÉO Hub par patient_id + study_uid
#[tauri::command]
async fn teo_fetch_report(patient_id: String, study_uid: String) -> Result<teo_client::models::TeoAiReportResponse, String> {
    teo_client::fetch_ai_report(&patient_id, &study_uid).await.map_err(|e| e.to_string())
}

/// Envoie un rapport validé à TÉO Hub
#[tauri::command]
async fn teo_submit_approved(
    patient_id: String,
    study_uid: String,
    approved_report: String,
) -> Result<teo_client::models::TeoApprovalResponse, String> {
    let report = teo_client::models::TeoApprovedReport {
        patient_id,
        study_uid,
        approved_report,
    };
    
    teo_client::submit_approved_report(report).await.map_err(|e| e.to_string())
}

/// Récupère la configuration TÉO Hub actuelle (sans secrets)
#[tauri::command]
fn teo_get_config() -> teo_client::models::TeoHubConfigInfo {
    let cfg = config::get_config();
    teo_client::models::TeoHubConfigInfo {
        enabled: cfg.teo_hub.enabled,
        host: cfg.teo_hub.host.clone(),
        port: cfg.teo_hub.port,
        tls_enabled: cfg.teo_hub.tls_enabled,
        timeout_secs: cfg.teo_hub.timeout_secs,
        retry_count: cfg.teo_hub.retry_count,
        has_api_token: !cfg.teo_hub.api_token.is_empty(),
        has_tls_certs: !cfg.teo_hub.cert_file.is_empty() && !cfg.teo_hub.key_file.is_empty(),
    }
}

/// Récupère le statut de connexion TÉO Hub
#[tauri::command]
fn teo_get_connection_status() -> String {
    format!("{:?}", teo_client::get_connection_status())
}

// 🔗 EXTRACTION DU TID DEPUIS UNE DEEP LINK
// Formats supportés:
//   - airadcr://open?tid=ABC123
//   - airadcr://open/ABC123
//   - airadcr://ABC123
fn extract_tid_from_deep_link(url: &str) -> Option<String> {
    // Retirer le préfixe airadcr://
    let path = url.strip_prefix("airadcr://")?;
    
    // 🛡️ SÉCURITÉ: Fonction de validation du tid
    let validate_tid = |tid: &str| -> Option<String> {
        // Limite de longueur (protection DoS)
        if tid.is_empty() || tid.len() > 64 {
            if tid.len() > 64 {
                log::warn!("[Deep Link] tid trop long rejeté: {} chars", tid.len());
            }
            return None;
        }
        // Validation caractères autorisés (alphanumériques, tirets, underscores)
        if tid.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            Some(tid.to_string())
        } else {
            log::warn!("[Deep Link] tid avec caractères invalides rejeté");
            None
        }
    };
    
    // Format: airadcr://open?tid=ABC123
    if path.contains("tid=") {
        let tid = path.split("tid=").nth(1)?;
        let tid = tid.split('&').next()?; // Au cas où il y a d'autres params
        return validate_tid(tid);
    }
    
    // Format: airadcr://open/ABC123
    if path.starts_with("open/") {
        let tid = path.strip_prefix("open/")?;
        return validate_tid(tid);
    }
    
    // Format: airadcr://ABC123 (direct)
    let tid = path.trim_start_matches("open").trim_start_matches('/').trim_start_matches('?');
    validate_tid(tid)
}

// 🔗 TRAITEMENT DES DEEP LINKS AU PREMIER LANCEMENT
fn process_initial_deep_link(app: &tauri::App) {
    // Récupérer les arguments de ligne de commande
    let args: Vec<String> = std::env::args().collect();
    debug!("[Deep Link] Arguments de démarrage: {:?}", args);
    
    for arg in args.iter().skip(1) { // Skip le nom de l'exe
        // Deep Link: airadcr://...
        if arg.starts_with("airadcr://") {
            if let Some(tid) = extract_tid_from_deep_link(arg) {
                info!("[Deep Link] Premier lancement avec tid: {}", tid);
                if let Some(window) = app.get_window("main") {
                    // Délai pour laisser le temps à la fenêtre de se charger
                    let window_clone = window.clone();
                    let tid_clone = tid.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(1500));
                        // Émettre uniquement le tid, pas l'URL complète
                        let _ = window_clone.emit("airadcr:navigate_to_report", &tid_clone);
                        info!("[Deep Link] Navigation émise vers tid={}", tid_clone);
                    });
                }
                return;
            }
        }
        // CLI: --open-tid=...
        if arg.starts_with("--open-tid=") {
            let tid = arg.trim_start_matches("--open-tid=");
            info!("[CLI] Premier lancement avec tid: {}", tid);
            if let Some(window) = app.get_window("main") {
                let window_clone = window.clone();
                let tid_string = tid.to_string();
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(1500));
                    // Émettre uniquement le tid, pas l'URL complète
                    let _ = window_clone.emit("airadcr:navigate_to_report", &tid_string);
                    info!("[CLI] Navigation émise vers tid={}", tid_string);
                });
            }
            return;
        }
    }
}

fn main() {
    // 🆕 Initialiser le système de logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("Démarrage de AIRADCR Desktop v{}", env!("CARGO_PKG_VERSION"));
    
    #[cfg(target_os = "windows")]
    enable_dpi_awareness();
    
    // 🌐 Démarrer le serveur HTTP local dans un thread séparé
    let app_data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("airadcr-desktop");
    
    let db = match database::Database::new(app_data_dir.clone()) {
        Ok(db) => {
            info!("[Database] Initialisée avec succès");
            Arc::new(db)
        }
        Err(e) => {
            error!("[Database] Erreur d'initialisation: {}", e);
            // Continuer sans base de données (le serveur HTTP ne fonctionnera pas)
            std::process::exit(1);
        }
    };
    
    // Clone pour le serveur HTTP
    let db_for_server = Arc::clone(&db);
    
    // Démarrer le serveur HTTP dans un thread séparé
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            if let Err(e) = http_server::start_server(8741, db_for_server).await {
                error!("[HTTP Server] Erreur: {}", e);
            }
        });
    });
    
    // Clone pour le cleanup périodique et backup
    let db_for_cleanup = Arc::clone(&db);
    let db_path = app_data_dir.join("airadcr.db");
    
    // 🧹 Démarrer le cleanup automatique des rapports expirés + backup quotidien (toutes les 10 minutes)
    thread::spawn(move || {
        use std::sync::atomic::{AtomicU64, Ordering};
        use crate::database::backup::BackupManager;
        
        // Compteur pour backup quotidien (1 jour = 144 cycles de 10 min)
        static BACKUP_COUNTER: AtomicU64 = AtomicU64::new(0);
        let backup_manager = BackupManager::new(db_path, 7); // 7 jours de rétention
        
        // Backup initial au démarrage
        match backup_manager.create_backup() {
            Ok(path) => info!("[Backup] Backup initial créé: {:?}", path),
            Err(e) => warn!("[Backup] Erreur backup initial: {}", e),
        }
        
        loop {
            thread::sleep(Duration::from_secs(600)); // 10 minutes
            
            // Cleanup des rapports expirés
            match db_for_cleanup.cleanup_expired_reports() {
                Ok(count) if count > 0 => {
                    info!("[Cleanup] {} rapport(s) expiré(s) supprimé(s)", count);
                }
                Ok(_) => {} // Rien à nettoyer
                Err(e) => error!("[Cleanup] Erreur: {}", e),
            }
            
            // Backup quotidien (toutes les 144 cycles = 24h)
            let counter = BACKUP_COUNTER.fetch_add(1, Ordering::SeqCst);
            if counter % 144 == 0 && counter > 0 {
                match backup_manager.create_backup() {
                    Ok(path) => {
                        info!("[Backup] Backup quotidien créé: {:?}", path);
                        // Nettoyer les anciens backups
                        if let Ok(deleted) = backup_manager.cleanup_old_backups() {
                            if deleted > 0 {
                                info!("[Backup] {} ancien(s) backup(s) supprimé(s)", deleted);
                            }
                        }
                    }
                    Err(e) => error!("[Backup] Erreur backup quotidien: {}", e),
                }
            }
        }
    });
    
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
        // 🔒 Protection contre les instances multiples + Deep Links
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            debug!("[Single Instance] Instance secondaire détectée");
            debug!("   Arguments: {:?}", argv);
            
            // Focus la fenêtre existante
            if let Some(window) = app.get_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                debug!("[Single Instance] Fenêtre principale focusée");
            }
            
            // Traiter les arguments pour deep links et --open-tid
            for arg in argv.iter() {
                // Deep Link: airadcr://open?tid=ABC123
                if arg.starts_with("airadcr://") {
                    if let Some(tid) = extract_tid_from_deep_link(arg) {
                        info!("[Deep Link] Navigation vers tid: {}", tid);
                        if let Some(window) = app.get_window("main") {
                            // Émettre uniquement le tid
                            let _ = window.emit("airadcr:navigate_to_report", &tid);
                        }
                    }
                    break;
                }
                // Argument classique: --open-tid=ABC123
                if arg.starts_with("--open-tid=") {
                    let tid = arg.trim_start_matches("--open-tid=");
                    info!("[CLI] Navigation vers tid: {}", tid);
                    if let Some(window) = app.get_window("main") {
                        // Émettre uniquement le tid
                        let _ = window.emit("airadcr:navigate_to_report", tid);
                    }
                    break;
                }
            }
        }))
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
                        Err(e) => error!("Error checking window visibility: {}", e),
                    }
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
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
                            warn!("Mutex poisoned in tray event, recovering...");
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
                    error!("Error hiding window on close: {}", e);
                }
                api.prevent_close();
            }
            WindowEvent::Destroyed => {
                // 🔒 Lock file désactivé temporairement
                // remove_lock_file();
                // println!("🔓 Fichier de verrouillage supprimé");
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
            has_text_selection,
            perform_injection_at_position,
            perform_injection,
            set_ignore_cursor_events,
            perform_injection_at_position_direct,
            get_active_window_info,
            get_window_at_point,
            simulate_key_in_iframe,
            get_virtual_desktop_info,
            get_physical_window_rect,
            get_window_client_rect_at_point,
            write_log,
            get_log_path,
            open_log_folder,
            // 🆕 Commandes Debug Panel - Base de données
            get_all_pending_reports,
            get_api_keys_list,
            get_database_stats,
            cleanup_expired_reports_cmd,
            delete_pending_report_cmd,
            create_api_key_cmd,
            revoke_api_key_cmd,
            // 🆕 Commandes Access Logs (AUDIT)
            get_access_logs,
            get_access_logs_stats,
            cleanup_access_logs,
            // 🆕 Commandes TÉO Hub Client
            teo_check_health,
            teo_fetch_report,
            teo_submit_approved,
            teo_get_config,
            teo_get_connection_status,
            // 🎤 Commandes SpeechMike natif (HID USB)
            speechmike_get_status,
            speechmike_list_devices,
            speechmike_set_led
        ])
        .setup(|app| {
            debug!("[DEBUG] .setup() appelé - enregistrement raccourcis SpeechMike");
            let tx = register_global_shortcuts(app.handle());
            
            // 🌐 Stocker l'AppHandle pour le serveur HTTP
            let _ = APP_HANDLE.set(app.handle());
            info!("[Global] AppHandle stocké pour communication HTTP → Tauri");
            
            // 🔗 Traiter les deep links au premier lancement
            process_initial_deep_link(app);
            
            // 🎤 Démarrer le thread de détection SpeechMike natif (HID USB)
            let sm_state = std::sync::Arc::new(speechmike::SpeechMikeState::default());
            app.manage(sm_state.clone());
            speechmike::start_speechmike_thread(tx, sm_state, app.handle());
            info!("[SpeechMike] Thread de détection HID natif lancé");
            
            // 🎯 Assertion Always-on-top après stabilisation WebView2
            if let Some(window) = app.get_window("main") {
                let window_clone = window.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(800));
                    let _ = window_clone.set_always_on_top(true);
                    debug!("Always-on-top: Assertion initiale (800ms)");
                });
            }
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    // ✅ Utilisation idiomatique de Tauri - pas de custom logic dans .run()
    app.run(|_app_handle, _event| {
        // Handle app events if needed
    });
}

// ✅ Raccourcis globaux — Pattern officiel Tauri: channel tokio pour dispatch thread-safe
// Corrige le problème fondamental : les callbacks GlobalShortcutManager s'exécutent dans un thread
// secondaire où window.eval() (COM WebView2) et SetForegroundWindow() (UIPI) échouent silencieusement.
// Solution : tx.send() non-bloquant → tokio task → window.emit() depuis le bon runtime.
// 🆕 Retourne le tx sender pour que le thread SpeechMike puisse utiliser le même canal
fn register_global_shortcuts(app_handle: tauri::AppHandle) -> tokio::sync::mpsc::UnboundedSender<&'static str> {
    // Channel tokio : bridge thread secondaire → runtime Tauri async
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<&'static str>();

    // Task tokio : reçoit les actions et émet les événements Tauri depuis le bon contexte
    let handle_for_task = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(action) = rx.recv().await {
            if let Some(window) = handle_for_task.get_window("main") {
                match action {
                    "toggle_recording" => {
                        debug!("[Shortcuts/task] Émission airadcr:dictation_startstop");
                        window.emit("airadcr:dictation_startstop", ()).ok();
                    }
                    "toggle_pause" => {
                        debug!("[Shortcuts/task] Émission airadcr:dictation_pause");
                        window.emit("airadcr:dictation_pause", ()).ok();
                    }
                    "inject_raw" => {
                        debug!("[Shortcuts/task] Émission airadcr:inject_raw");
                        window.emit("airadcr:inject_raw", ()).ok();
                    }
                    "inject_structured" => {
                        debug!("[Shortcuts/task] Émission airadcr:inject_structured");
                        window.emit("airadcr:inject_structured", ()).ok();
                    }
                    _ => {}
                }
            }
        }
    });

    let mut shortcut_manager = app_handle.global_shortcut_manager();

    // 🎨 DEBUG PANEL: Ctrl+Alt+D
    let handle_debug = app_handle.clone();
    shortcut_manager
        .register("Ctrl+Alt+D", move || {
            debug!("[Shortcuts] Ctrl+Alt+D pressé (debug panel)");
            if let Some(window) = handle_debug.get_window("main") {
                window.emit("airadcr:toggle_debug", ()).ok();
            }
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Alt+D: {}", e));

    // 📋 LOG WINDOW: Ctrl+Alt+L
    let handle_logs = app_handle.clone();
    shortcut_manager
        .register("Ctrl+Alt+L", move || {
            debug!("[Shortcuts] Ctrl+Alt+L pressé (log window)");
            if let Some(window) = handle_logs.get_window("main") {
                window.emit("airadcr:toggle_logs", ()).ok();
            }
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Alt+L: {}", e));

    // 🧪 TEST INJECTION: Ctrl+Alt+I
    let handle_test = app_handle.clone();
    shortcut_manager
        .register("Ctrl+Alt+I", move || {
            debug!("[Shortcuts] Ctrl+Alt+I pressé (test injection)");
            if let Some(window) = handle_test.get_window("main") {
                window.emit("airadcr:test_injection", ()).ok();
            }
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Alt+I: {}", e));

    // 🎤 DICTATION: Ctrl+Shift+D (Start/Stop dictée)
    let tx_d = tx.clone();
    shortcut_manager
        .register("Ctrl+Shift+D", move || {
            debug!("[Shortcuts] Ctrl+Shift+D pressé → tx.send(toggle_recording)");
            let _ = tx_d.send("toggle_recording");
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Shift+D: {}", e));

    // 🎤 DICTATION: Ctrl+Shift+P (Pause/Resume dictée)
    let tx_p = tx.clone();
    shortcut_manager
        .register("Ctrl+Shift+P", move || {
            debug!("[Shortcuts] Ctrl+Shift+P pressé → tx.send(toggle_pause)");
            let _ = tx_p.send("toggle_pause");
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Shift+P: {}", e));

    // 💉 INJECTION: Ctrl+Shift+T (Inject texte brut)
    let tx_t = tx.clone();
    shortcut_manager
        .register("Ctrl+Shift+T", move || {
            debug!("[Shortcuts] Ctrl+Shift+T pressé → tx.send(inject_raw)");
            let _ = tx_t.send("inject_raw");
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Shift+T: {}", e));

    // 💉 INJECTION: Ctrl+Shift+S (Inject rapport structuré)
    let tx_s = tx.clone();
    shortcut_manager
        .register("Ctrl+Shift+S", move || {
            debug!("[Shortcuts] Ctrl+Shift+S pressé → tx.send(inject_structured)");
            let _ = tx_s.send("inject_structured");
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Shift+S: {}", e));

    // 🎯 ERGONOMIC: Ctrl+Space → toggle_recording (style Wispr Flow / SuperWhisper)
    // Touche la plus accessible : auriculaire Ctrl + pouce Space, une seule main
    let tx_space = tx.clone();
    shortcut_manager
        .register("Ctrl+Space", move || {
            debug!("[Shortcuts] Ctrl+Space pressé (ergonomic) → toggle_recording");
            let _ = tx_space.send("toggle_recording");
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Space: {}", e));

    // 🎯 ERGONOMIC: Ctrl+Shift+Space → toggle_pause
    let tx_shift_space = tx.clone();
    shortcut_manager
        .register("Ctrl+Shift+Space", move || {
            debug!("[Shortcuts] Ctrl+Shift+Space pressé (ergonomic) → toggle_pause");
            let _ = tx_shift_space.send("toggle_pause");
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement Ctrl+Shift+Space: {}", e));

    // ANTI-GHOST: F9 (désactiver click-through)
    let handle_f9 = app_handle.clone();
    shortcut_manager
        .register("F9", move || {
            debug!("[Shortcuts] F9 pressé (anti-fantôme)");
            if let Some(window) = handle_f9.get_window("main") {
                window.emit("airadcr:force_clickable", ()).ok();
            }
        })
        .unwrap_or_else(|e| warn!("Erreur enregistrement F9: {}", e));

    info!("[Shortcuts] Raccourcis globaux enregistrés (channel tokio): Ctrl+Alt+D/L/I, F9, Ctrl+Shift+D/P/T/S, Ctrl+Space, Ctrl+Shift+Space");
    
    tx // 🆕 Retourner le sender pour le thread SpeechMike
}