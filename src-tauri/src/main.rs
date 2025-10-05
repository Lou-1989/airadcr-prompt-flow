#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::{Mutex, Arc};
use tauri::{CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu, WindowEvent, GlobalShortcutManager};
use serde::{Deserialize, Serialize};
use enigo::{Enigo, Button, Key, Settings, Direction, Coordinate, Mouse, Keyboard};
use arboard::Clipboard;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use active_win_pos_rs::get_active_window;
use log::{info, warn, error, debug};
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
extern crate chrono;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};

// 🆕 Fonction pour activer DPI Per-Monitor V2 (multi-écrans + multi-DPI)
#[cfg(target_os = "windows")]
fn enable_dpi_awareness() {
    use winapi::um::winuser::SetProcessDpiAwarenessContext;
    use winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2;
    
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) == 0 {
            eprintln!("⚠️  Échec activation DPI Per-Monitor V2, fallback mode par défaut");
        } else {
            println!("✅ DPI Per-Monitor V2 activé (coordonnées physiques multi-écrans)");
        }
    }
}

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
    // Retry logic pour gérer les erreurs temporaires multi-écrans
    let mut retry_count = 0;
    let max_retries = 3;
    
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
            Err(e) if retry_count < max_retries - 1 => {
                retry_count += 1;
                thread::sleep(Duration::from_millis(100));
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

// 🆕 DÉTECTION DE SÉLECTION DE TEXTE
#[tauri::command]
async fn has_text_selection(state: State<'_, AppState>) -> Result<bool, String> {
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    // Sauvegarder le clipboard actuel
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original = clipboard.get_text().unwrap_or_default();
    
    // Simuler Ctrl+C pour copier la sélection (si elle existe)
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('c'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    
    // Vérifier si le clipboard a changé
    let after_copy = clipboard.get_text().unwrap_or_default();
    let has_selection = !after_copy.is_empty() && after_copy != original;
    
    // Restaurer le clipboard original
    if original != after_copy {
        clipboard.set_text(&original).map_err(|e| e.to_string())?;
    }
    
    println!("🔍 Détection sélection: {}", if has_selection { "OUI" } else { "NON" });
    Ok(has_selection)
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

#[cfg(target_os = "windows")]
use winapi::um::winuser::{SetCursorPos, SendInput, INPUT, INPUT_MOUSE, INPUT_KEYBOARD, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, KEYEVENTF_KEYUP};
#[cfg(target_os = "windows")]
use winapi::um::winuser::{VK_CONTROL, KEYEVENTF_SCANCODE};
#[cfg(target_os = "windows")]
use winapi::um::winuser::{WindowFromPoint, GetAncestor, SetForegroundWindow, GA_ROOT, GetForegroundWindow, GetWindowRect, IsIconic, ShowWindow, GetWindowPlacement, SetWindowPlacement, SW_RESTORE, SW_SHOWNORMAL};
#[cfg(target_os = "windows")]
use winapi::shared::windef::{POINT, RECT};
#[cfg(target_os = "windows")]
use winapi::um::winuser::WINDOWPLACEMENT;

// 🆕 INJECTION WINDOWS ROBUSTE avec Win32 API pour multi-écrans
#[tauri::command]
async fn perform_injection_at_position_direct(x: i32, y: i32, text: String, state: State<'_, AppState>) -> Result<(), String> {
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    println!("🎯 [Injection Rust] Début - Position: ({}, {}) - Texte: {} chars", x, y, text.len());
    let start_time = std::time::Instant::now();
    
    println!("🎯 [Multi-écrans] Injection à ({}, {}) - {} caractères", x, y, text.len());
    
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
            println!("⚠️  [Multi-écrans] Coordonnées clampées: ({}, {}) → ({}, {}) [Bureau: ({}, {}) {}x{}]", 
                x, y, cx, cy, vd_x, vd_y, vd_width, vd_height);
        }
        
        (cx, cy)
    };
    
    #[cfg(not(target_os = "windows"))]
    let (clamped_x, clamped_y) = (x, y);
    
    if clamped_x < 0 || clamped_y < 0 {
        println!("⚠️  [Multi-écrans] Coordonnées négatives détectées (écran secondaire gauche/haut)");
    }
    
    // 🆕 LOG: Fenêtre active AVANT clic
    match get_active_window() {
        Ok(win) => println!("📊 [Avant clic] Fenêtre active: {} ({})", win.app_name, win.title),
        Err(_) => println!("⚠️  [Avant clic] Impossible de récupérer la fenêtre active")
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
        println!("✅ [Win32] SetCursorPos({}, {}) réussi", clamped_x, clamped_y);
        
        // 🆕 FORCER LE FOCUS sur la fenêtre sous le curseur (multi-écrans)
        unsafe {
            let point = POINT { x: clamped_x, y: clamped_y };
            let hwnd = WindowFromPoint(point);
            
            if !hwnd.is_null() {
                let root_hwnd = GetAncestor(hwnd, GA_ROOT);
                if !root_hwnd.is_null() {
                    // ✨ DÉTECTION 1/3: Vérifier si la fenêtre est minimisée
                    if IsIconic(root_hwnd) != 0 {
                        println!("🔄 [Restauration] Fenêtre minimisée détectée, restauration...");
                        ShowWindow(root_hwnd, SW_RESTORE);
                        thread::sleep(Duration::from_millis(200)); // Attendre animation
                        println!("✅ [Restauration] Fenêtre restaurée depuis l'état minimisé");
                    }
                    
                    // ✨ DÉTECTION 2/3: Vérifier si la fenêtre est hors écran
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
                            println!("🔄 [Restauration] Fenêtre hors écran détectée: ({}, {}) - ({}, {})", 
                                rect.left, rect.top, rect.right, rect.bottom);
                            println!("   Bureau virtuel: ({}, {}) - ({}, {})", vd_x, vd_y, vd_right, vd_bottom);
                            
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
                                println!("✅ [Restauration] Fenêtre repositionnée à ({}, {})", new_x, new_y);
                                thread::sleep(Duration::from_millis(150));
                            } else {
                                println!("⚠️  [Restauration] SetWindowPlacement a échoué");
                            }
                        }
                    }
                    
                    // ✨ DÉTECTION 3/3: Focus final avec vérification
                    if SetForegroundWindow(root_hwnd) != 0 {
                        println!("✅ [Multi-écrans] Focus forcé sur fenêtre à ({}, {})", clamped_x, clamped_y);
                    } else {
                        println!("⚠️  [Multi-écrans] SetForegroundWindow a échoué");
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
    
    println!("🎯 [Injection] Position finale: ({}, {}) - SANS CLIC (préserve sélection)", clamped_x, clamped_y);
    thread::sleep(Duration::from_millis(10));
    
    // Créer enigo pour Ctrl+V
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    
    // 🆕 LOG: Fenêtre active AVANT Ctrl+V
    match get_active_window() {
        Ok(win) => println!("📊 [Avant Ctrl+V] Fenêtre active: {} ({})", win.app_name, win.title),
        Err(_) => println!("⚠️  [Avant Ctrl+V] Impossible de récupérer la fenêtre active")
    }
    
    // ✅ SAUVEGARDE du clipboard original
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let original_clipboard = clipboard.get_text().unwrap_or_default();
    println!("💾 Clipboard sauvegardé : {} caractères", original_clipboard.len());
    
    // Injection via Ctrl+V (remplace la sélection manuelle utilisateur si présente)
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    println!("📋 Texte copié dans clipboard : {} caractères", text.len());
    thread::sleep(Duration::from_millis(10));
    
    println!("⌨️  Envoi Ctrl+V (va remplacer la sélection si présente, sinon coller)");
    enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?;
    enigo.key(Key::Unicode('v'), Direction::Click).map_err(|e| e.to_string())?;
    enigo.key(Key::Control, Direction::Release).map_err(|e| e.to_string())?;
    
    thread::sleep(Duration::from_millis(50));
    println!("✅ Injection Ctrl+V terminée");
    
    // ✅ RESTAURATION du clipboard original
    if !original_clipboard.is_empty() {
        clipboard.set_text(&original_clipboard).map_err(|e| e.to_string())?;
        println!("✅ Clipboard restauré ({} caractères)", original_clipboard.len());
    }
    
    println!("✅ Injection Ctrl+V réussie ({} caractères)", text.len());
    println!("✅ [Injection Rust] Terminée avec succès en {}ms", start_time.elapsed().as_millis());
    
    Ok(())
}

// 🆕 Commande: Récupérer la fenêtre sous le curseur (même si AirADCR a le focus)
#[tauri::command]
async fn get_window_at_point(x: i32, y: i32) -> Result<WindowInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use winapi::shared::windef::POINT;
        use winapi::um::processthreadsapi::OpenProcess;
        use winapi::um::psapi::GetModuleFileNameExW;
        use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
        
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
            
            // ✅ CORRECTION CRITIQUE: Récupérer les infos de la fenêtre trouvée (pas get_active_window)
            
            // 1️⃣ Récupérer le titre de la fenêtre
            let mut title_buffer = vec![0u16; 256];
            let title_len = GetWindowTextW(root_hwnd, title_buffer.as_mut_ptr(), 256);
            let title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);
            
            // 2️⃣ Récupérer le nom du processus
            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(root_hwnd, &mut process_id);
            
            let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);
            let app_name = if !process_handle.is_null() {
                let mut process_name = vec![0u16; 260];
                let size = GetModuleFileNameExW(
                    process_handle, 
                    std::ptr::null_mut(), 
                    process_name.as_mut_ptr(), 
                    260
                );
                CloseHandle(process_handle);
                
                if size > 0 {
                    let full_path = String::from_utf16_lossy(&process_name[..size as usize]);
                    full_path.split('\\').last().unwrap_or("Unknown").to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };
            
            // 3️⃣ Récupérer le rectangle de la fenêtre
            let mut rect: RECT = std::mem::zeroed();
            if GetWindowRect(root_hwnd, &mut rect) == 0 {
                return Err("Impossible de récupérer les dimensions de la fenêtre".to_string());
            }
            
            Ok(WindowInfo {
                title,
                app_name,
                x: rect.left,
                y: rect.top,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
            })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("get_window_at_point non supporté sur cette plateforme".to_string())
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
            
            println!("🖥️  [Multi-écrans] Bureau virtuel: ({}, {}) {}x{} | DPI Per-Monitor V2: ACTIVÉ", x, y, width, height);
            
            Ok(VirtualDesktopInfo { x, y, width, height })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("get_virtual_desktop_info: Supporté uniquement sur Windows".to_string())
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
            
            println!("📐 [DPI] GetWindowRect physique: ({}, {}) {}x{}", 
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
        Err("get_physical_window_rect supporté uniquement sur Windows".to_string())
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
            
            println!("📐 [ClientRect] Fenêtre: {} ({}, {}) {}x{} [HWND: {:?}]", 
                app_name, window_rect.left, window_rect.top, window_width, window_height, root_hwnd);
            println!("📐 [ClientRect] Zone client: ({}, {}) {}x{}", 
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
        Err("get_window_client_rect_at_point supporté uniquement sur Windows".to_string())
    }
}

// 🎤 Commande: Simuler une touche dans l'iframe airadcr.com
#[tauri::command]
async fn simulate_key_in_iframe(window: tauri::Window, key: String) -> Result<(), String> {
    let key_code = get_key_code(&key);
    
    if key_code == 0 {
        return Err(format!("Touche non supportée: {}", key));
    }
    
    println!("🎤 [SpeechMike] Injection touche {} (code: {}) dans iframe", key, key_code);
    
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
    // Obtenir le chemin du dossier de logs dans AppData
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let log_dir = format!("{}\\AIRADCR\\logs", app_data);
    
    // Créer le dossier si nécessaire
    if let Err(e) = create_dir_all(&log_dir) {
        return Err(format!("Impossible de créer le dossier de logs: {}", e));
    }
    
    // Créer/ouvrir le fichier de log avec append
    let log_path = format!("{}\\app.log", log_dir);
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
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let log_path = format!("{}\\AIRADCR\\logs\\app.log", app_data);
    Ok(log_path)
}

// 🆕 COMMANDE POUR OUVRIR LE DOSSIER DES LOGS
#[tauri::command]
async fn open_log_folder() -> Result<(), String> {
    let app_data = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let log_dir = format!("{}\\AIRADCR\\logs", app_data);
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&log_dir)
            .spawn()
            .map_err(|e| format!("Impossible d'ouvrir le dossier: {}", e))?;
    }
    
    Ok(())
}

fn main() {
    // 🆕 Initialiser le système de logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
    
    info!("🚀 Démarrage de AIRADCR Desktop v1.0.0");
    
    #[cfg(target_os = "windows")]
    enable_dpi_awareness();
    
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
            open_log_folder
        ])
        .setup(|app| {
            // 🎤 Enregistrement des raccourcis globaux SpeechMike
            let app_handle = app.handle();
            let mut shortcut_manager = app.global_shortcut_manager();
            
            // F10: Démarrer/Reprendre dictée
            let handle_f10 = app_handle.clone();
            shortcut_manager
                .register("F10", move || {
                    if let Some(window) = handle_f10.get_window("main") {
                        let window_clone = window.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = simulate_key_in_iframe(window_clone, "F10".to_string()).await;
                        });
                    }
                })
                .unwrap_or_else(|e| eprintln!("❌ Erreur enregistrement F10: {}", e));
            
            // F11: Pause dictée
            let handle_f11 = app_handle.clone();
            shortcut_manager
                .register("F11", move || {
                    if let Some(window) = handle_f11.get_window("main") {
                        let window_clone = window.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = simulate_key_in_iframe(window_clone, "F11".to_string()).await;
                        });
                    }
                })
                .unwrap_or_else(|e| eprintln!("❌ Erreur enregistrement F11: {}", e));
            
            // F12: Terminer dictée (uniquement en production)
            #[cfg(not(debug_assertions))]
            {
                let handle_f12 = app_handle.clone();
                shortcut_manager
                    .register("F12", move || {
                        if let Some(window) = handle_f12.get_window("main") {
                            let window_clone = window.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = simulate_key_in_iframe(window_clone, "F12".to_string()).await;
                            });
                        }
                    })
                    .unwrap_or_else(|e| eprintln!("❌ Erreur enregistrement F12: {}", e));
                
                println!("✅ [SpeechMike] Raccourcis globaux enregistrés: F10 (Démarrer/Reprendre), F11 (Pause), F12 (Terminer)");
            }
            
            #[cfg(debug_assertions)]
            println!("⚠️ [DEV] F12 non enregistré (disponible pour DevTools)");
            
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Ready = event {
            if let Some(window) = app_handle.get_window("main") {
                let window_clone = window.clone();
                
                // 🎯 Assertion UNIQUE après stabilisation WebView2 (architecture événementielle)
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(800));
                    let _ = window_clone.set_always_on_top(true);
                    println!("✅ Always-on-top: Assertion initiale (800ms)");
                });
            }
        }
    });
}