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

#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetSystemMetrics, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN};

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

#[cfg(target_os = "windows")]
use winapi::um::winuser::{SetCursorPos, SendInput, INPUT, INPUT_MOUSE, INPUT_KEYBOARD, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, KEYEVENTF_KEYUP};
#[cfg(target_os = "windows")]
use winapi::um::winuser::{VK_CONTROL, KEYEVENTF_SCANCODE};

// 🆕 INJECTION WINDOWS ROBUSTE avec Win32 API pour multi-écrans
#[tauri::command]
async fn perform_injection_at_position_direct(text: String, x: i32, y: i32, state: State<'_, AppState>) -> Result<(), String> {
    let _clipboard_guard = match state.clipboard_lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Clipboard mutex poisoned, recovering...");
            poisoned.into_inner()
        }
    };
    
    println!("🎯 [Multi-écrans] Injection à ({}, {}) - {} caractères", x, y, text.len());
    if x < 0 || y < 0 {
        println!("⚠️  [Multi-écrans] Coordonnées négatives détectées (écran secondaire gauche/haut)");
    }
    
    thread::sleep(Duration::from_millis(10));
    
    // 🆕 WINDOWS: Utiliser SetCursorPos (Win32) pour multi-écrans robuste
    #[cfg(target_os = "windows")]
    {
        unsafe {
            if SetCursorPos(x, y) == 0 {
                return Err("Échec SetCursorPos (Win32)".to_string());
            }
        }
        println!("✅ [Win32] SetCursorPos({}, {}) réussi", x, y);
    }
    
    // FALLBACK: Autres OS utilisent Enigo
    #[cfg(not(target_os = "windows"))]
    {
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
        enigo.move_mouse(x, y, Coordinate::Abs).map_err(|e| e.to_string())?;
    }
    
    thread::sleep(Duration::from_millis(10));
    
    // Clic gauche (Enigo pour compatibilité)
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
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
            
            println!("🖥️  [Multi-écrans] Bureau virtuel: ({}, {}) {}x{}", x, y, width, height);
            
            Ok(VirtualDesktopInfo { x, y, width, height })
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("get_virtual_desktop_info: Supporté uniquement sur Windows".to_string())
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
            get_active_window_info,
            simulate_key_in_iframe,
            get_virtual_desktop_info
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
            
            // F12: Terminer dictée
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