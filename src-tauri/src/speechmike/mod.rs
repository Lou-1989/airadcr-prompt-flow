// 🎤 SpeechMike Native HID Module
// Direct USB HID communication without SpeechControl dependency
// Based on Google ChromeLabs dictation_support SDK mappings

pub mod devices;

use devices::*;
use hidapi::HidApi;
use log::{info, warn, error, debug};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use serde::Serialize;

/// Status of the SpeechMike connection
#[derive(Debug, Clone, Serialize)]
pub struct SpeechMikeStatus {
    pub connected: bool,
    pub device_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

impl Default for SpeechMikeStatus {
    fn default() -> Self {
        Self {
            connected: false,
            device_name: "Aucun périphérique détecté".to_string(),
            vendor_id: 0,
            product_id: 0,
        }
    }
}

/// Actions that the SpeechMike thread sends to the main Tauri runtime
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpeechMikeAction {
    ToggleRecording,
    TogglePause,
    InjectRaw,
    InjectStructured,
    FinalizeAndInject,
}

/// Shared state for the SpeechMike module
pub struct SpeechMikeState {
    pub status: Arc<Mutex<SpeechMikeStatus>>,
    pub running: Arc<AtomicBool>,
}

impl Default for SpeechMikeState {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(SpeechMikeStatus::default())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }
}

/// Map a decoded ButtonEvent to our application action
fn button_to_action(button: ButtonEvent) -> Option<SpeechMikeAction> {
    match button {
        ButtonEvent::Record   => Some(SpeechMikeAction::ToggleRecording),
        ButtonEvent::Stop     => Some(SpeechMikeAction::TogglePause),
        ButtonEvent::Play     => Some(SpeechMikeAction::TogglePause),
        ButtonEvent::Instr    => Some(SpeechMikeAction::InjectRaw),
        ButtonEvent::F1A      => Some(SpeechMikeAction::InjectStructured),
        ButtonEvent::EolPrio  => Some(SpeechMikeAction::FinalizeAndInject),
        _ => None,
    }
}

/// Convert SpeechMikeAction to the string used by the tokio channel in main.rs
pub fn action_to_channel_str(action: SpeechMikeAction) -> &'static str {
    match action {
        SpeechMikeAction::ToggleRecording   => "toggle_recording",
        SpeechMikeAction::TogglePause       => "toggle_pause",
        SpeechMikeAction::InjectRaw         => "inject_raw",
        SpeechMikeAction::InjectStructured  => "inject_structured",
        SpeechMikeAction::FinalizeAndInject  => "inject_structured", // Map to structured for now
    }
}

/// Decode the HID input report into pressed ButtonEvents
/// Returns a bitmask of ButtonEvent values
fn decode_input_report(data: &[u8], is_pm4: bool) -> u32 {
    // Input reports: command byte at [0], button bitmask at bytes [7..8] (uint16 LE)
    if data.len() < 9 {
        return 0;
    }
    
    // Check command byte - only process button press events (0x80)
    if data[0] != HidCommand::ButtonPressEvent as u8 {
        return 0;
    }
    
    // Read uint16 little-endian from offset 7
    let input_bitmask = u16::from_le_bytes([data[7], data[8]]);
    
    let mappings = if is_pm4 {
        BUTTON_MAPPINGS_POWERMIC4
    } else {
        BUTTON_MAPPINGS_SPEECHMIKE
    };
    
    let mut output_bitmask: u32 = 0;
    for &(button_event, input_mask) in mappings {
        if input_bitmask & input_mask != 0 {
            output_bitmask |= button_event as u32;
        }
    }
    
    output_bitmask
}

/// Extract individual pressed buttons from a bitmask
fn extract_buttons(bitmask: u32) -> Vec<ButtonEvent> {
    let all_buttons = [
        ButtonEvent::Rewind, ButtonEvent::Play, ButtonEvent::Forward,
        ButtonEvent::InsOvr, ButtonEvent::Record, ButtonEvent::Command,
        ButtonEvent::Stop, ButtonEvent::Instr, ButtonEvent::F1A,
        ButtonEvent::F2B, ButtonEvent::F3C, ButtonEvent::F4D,
        ButtonEvent::EolPrio, ButtonEvent::Transcribe,
    ];
    
    all_buttons.iter()
        .filter(|b| bitmask & (**b as u32) != 0)
        .copied()
        .collect()
}

/// Start the SpeechMike HID polling thread
/// Sends actions through the provided tokio channel (same as global shortcuts)
pub fn start_speechmike_thread(
    tx: tokio::sync::mpsc::UnboundedSender<&'static str>,
    state: Arc<SpeechMikeState>,
    app_handle: tauri::AppHandle,
) {
    let running = state.running.clone();
    let status = state.status.clone();
    
    // Don't start if already running
    if running.load(Ordering::SeqCst) {
        warn!("[SpeechMike] Thread déjà en cours d'exécution");
        return;
    }
    
    running.store(true, Ordering::SeqCst);
    
    thread::spawn(move || {
        info!("[SpeechMike] Thread de détection HID démarré");
        
        // Debounce: track last action time per button
        let mut last_action_time = std::time::Instant::now();
        let debounce_ms = 150; // 150ms debounce between same actions
        let mut last_bitmask: u32 = 0;
        
        loop {
            if !running.load(Ordering::SeqCst) {
                info!("[SpeechMike] Thread arrêté (signal reçu)");
                break;
            }
            
            // Try to initialize HidApi
            let api = match HidApi::new() {
                Ok(api) => api,
                Err(e) => {
                    debug!("[SpeechMike] hidapi init error (retrying in 5s): {}", e);
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };
            
            // Scan for supported devices
            let mut found_device = None;
            for device_info in api.device_list() {
                let vid = device_info.vendor_id();
                let pid = device_info.product_id();
                
                if let Some(filter) = is_supported_device(vid, pid) {
                    info!("[SpeechMike] ✅ Périphérique détecté: {} (VID:{:04x} PID:{:04x})", 
                        filter.description, vid, pid);
                    found_device = Some((vid, pid, filter.description));
                    break;
                }
            }
            
            let (vid, pid, desc) = match found_device {
                Some(d) => d,
                None => {
                    // Update status: not connected
                    if let Ok(mut s) = status.lock() {
                        if s.connected {
                            s.connected = false;
                            s.device_name = "Aucun périphérique détecté".to_string();
                            s.vendor_id = 0;
                            s.product_id = 0;
                            // Emit disconnect event
                            let _ = app_handle.emit_all("airadcr:speechmike_disconnected", ());
                            info!("[SpeechMike] Périphérique déconnecté");
                        }
                    }
                    debug!("[SpeechMike] Aucun périphérique trouvé, nouvelle tentative dans 3s");
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };
            
            // Try to open the device
            let device = match api.open(vid, pid) {
                Ok(d) => d,
                Err(e) => {
                    warn!("[SpeechMike] ⚠️ Impossible d'ouvrir le périphérique (VID:{:04x} PID:{:04x}): {}", vid, pid, e);
                    warn!("[SpeechMike] → Possible conflit avec SpeechControl. Fallback sur raccourcis clavier.");
                    
                    if let Ok(mut s) = status.lock() {
                        s.connected = false;
                        s.device_name = format!("{} (verrouillé par un autre processus)", desc);
                    }
                    
                    thread::sleep(Duration::from_secs(10));
                    continue;
                }
            };
            
            // Successfully opened - update status
            if let Ok(mut s) = status.lock() {
                s.connected = true;
                s.device_name = desc.to_string();
                s.vendor_id = vid;
                s.product_id = pid;
            }
            
            // Emit connected event
            let connect_status = SpeechMikeStatus {
                connected: true,
                device_name: desc.to_string(),
                vendor_id: vid,
                product_id: pid,
            };
            let _ = app_handle.emit_all("airadcr:speechmike_connected", &connect_status);
            info!("[SpeechMike] 🎤 Connecté: {} (natif HID)", desc);
            
            // Set LED to idle (green)
            if let Err(e) = set_led_state(&device, SimpleLedState::Off) {
                debug!("[SpeechMike] LED control non supporté: {}", e);
            }
            
            let is_pm4 = is_powermic4(vid, pid);
            
            // Set non-blocking mode with timeout
            device.set_blocking_mode(false).ok();
            
            // Main polling loop for this device
            let mut buf = [0u8; 64];
            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                
                // Read with 10ms timeout
                match device.read_timeout(&mut buf, 10) {
                    Ok(0) => {
                        // No data (timeout) - normal, continue polling
                        continue;
                    }
                    Ok(n) => {
                        let bitmask = decode_input_report(&buf[..n], is_pm4);
                        
                        // Only process on state CHANGE (press/release detection)
                        if bitmask != last_bitmask {
                            let previously_pressed = last_bitmask;
                            last_bitmask = bitmask;
                            
                            // Detect newly pressed buttons (transition from 0 to 1)
                            let newly_pressed = bitmask & !previously_pressed;
                            
                            if newly_pressed != 0 {
                                let buttons = extract_buttons(newly_pressed);
                                let now = std::time::Instant::now();
                                
                                for button in buttons {
                                    // Debounce check
                                    if now.duration_since(last_action_time).as_millis() < debounce_ms as u128 {
                                        debug!("[SpeechMike] Debounce: {:?} ignoré", button);
                                        continue;
                                    }
                                    
                                    if let Some(action) = button_to_action(button) {
                                        let action_str = action_to_channel_str(action);
                                        debug!("[SpeechMike] 🎯 Bouton {:?} → action: {}", button, action_str);
                                        
                                        if let Err(e) = tx.send(action_str) {
                                            error!("[SpeechMike] Erreur envoi action: {}", e);
                                        }
                                        
                                        last_action_time = now;
                                    } else {
                                        debug!("[SpeechMike] Bouton {:?} non mappé", button);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // Device disconnected or error
                        warn!("[SpeechMike] ❌ Erreur lecture HID: {} - reconnexion...", e);
                        last_bitmask = 0;
                        
                        if let Ok(mut s) = status.lock() {
                            s.connected = false;
                            s.device_name = "Déconnecté".to_string();
                        }
                        let _ = app_handle.emit_all("airadcr:speechmike_disconnected", ());
                        
                        break; // Return to device scanning loop
                    }
                }
            }
        }
        
        info!("[SpeechMike] Thread terminé");
    });
}

/// Set the LED state on the device
pub fn set_led_state(device: &hidapi::HidDevice, state: SimpleLedState) -> Result<(), String> {
    let led_data = build_led_report(state);
    let mut report = vec![0u8; 10]; // Report ID 0 + command + 8 data bytes
    report[0] = 0; // Report ID
    report[1] = HidCommand::SetLed as u8;
    report[2..10].copy_from_slice(&led_data);
    
    device.write(&report).map_err(|e| format!("LED write error: {}", e))?;
    Ok(())
}

/// Stop the SpeechMike polling thread
pub fn stop_speechmike_thread(state: &SpeechMikeState) {
    state.running.store(false, Ordering::SeqCst);
    info!("[SpeechMike] Signal d'arrêt envoyé");
}
