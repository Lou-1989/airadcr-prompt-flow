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
use tauri::Manager;

/// Status of the SpeechMike connection
#[derive(Debug, Clone, Serialize)]
pub struct SpeechMikeStatus {
    pub connected: bool,
    pub device_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_code: Option<String>,
    pub has_slider: bool,
    pub event_mode: Option<String>,
}

impl Default for SpeechMikeStatus {
    fn default() -> Self {
        Self {
            connected: false,
            device_name: "Aucun périphérique détecté".to_string(),
            vendor_id: 0,
            product_id: 0,
            device_code: None,
            has_slider: false,
            event_mode: None,
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
/// Phase 2 fix: LED commands sent via channel to polling thread (no double-open)
pub struct SpeechMikeState {
    pub status: Arc<Mutex<SpeechMikeStatus>>,
    pub running: Arc<AtomicBool>,
    /// Channel to send LED commands to the polling thread (avoids double-open HID)
    pub led_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<SimpleLedState>>>>,
}

impl Default for SpeechMikeState {
    fn default() -> Self {
        Self {
            status: Arc::new(Mutex::new(SpeechMikeStatus::default())),
            running: Arc::new(AtomicBool::new(false)),
            led_tx: Arc::new(Mutex::new(None)),
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
        // PowerMic IV extra: TabBackward/TabForward currently unmapped
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
        SpeechMikeAction::FinalizeAndInject  => "inject_structured",
    }
}

/// Decode the HID input report into pressed ButtonEvents
/// Returns a bitmask of ButtonEvent values
/// Phase 1: Applies slider filtering when has_slider is true
fn decode_input_report(data: &[u8], is_pm4: bool, has_slider: bool) -> u32 {
    if data.len() < 9 {
        return 0;
    }
    
    // Check command byte - only process button press events (0x80)
    if data[0] != HidCommand::ButtonPressEvent as u8 {
        return 0;
    }
    
    // Read uint16 little-endian from offset 7
    let mut input_bitmask = u16::from_le_bytes([data[7], data[8]]);
    
    // Phase 1: Apply slider filter for models with a slider switch
    if has_slider && !is_pm4 {
        input_bitmask = filter_slider_bits(input_bitmask);
    }
    
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
        ButtonEvent::ScanEnd, ButtonEvent::Rewind, ButtonEvent::Play, ButtonEvent::Forward,
        ButtonEvent::InsOvr, ButtonEvent::Record, ButtonEvent::Command, ButtonEvent::ScanSuccess,
        ButtonEvent::Stop, ButtonEvent::Instr, ButtonEvent::F1A,
        ButtonEvent::F2B, ButtonEvent::F3C, ButtonEvent::F4D,
        ButtonEvent::EolPrio, ButtonEvent::Transcribe,
        ButtonEvent::TabBackward, ButtonEvent::TabForward,
    ];
    
    all_buttons.iter()
        .filter(|b| bitmask & (**b as u32) != 0)
        .copied()
        .collect()
}

/// Phase 2: Try to fetch device code via HID command sequence
/// Returns the device code string (e.g. "LFH3500") or None
fn fetch_device_code(device: &hidapi::HidDevice) -> Option<String> {
    // Step 1: Check if it's a SpeechMike Premium
    let mut cmd = [0u8; 10];
    cmd[0] = 0; // Report ID
    cmd[1] = HidCommand::IsSpeechMikePremium as u8;
    
    if device.write(&cmd).is_err() {
        debug!("[SpeechMike] Device code fetch: write failed");
        return None;
    }
    
    let mut buf = [0u8; 64];
    match device.read_timeout(&mut buf, 500) {
        Ok(n) if n > 1 && buf[0] == HidCommand::IsSpeechMikePremium as u8 => {
            let is_premium = buf[1] != 0;
            debug!("[SpeechMike] Is Premium: {}", is_premium);
            
            // Step 2: Get device code based on type
            let code_cmd = if is_premium {
                HidCommand::GetDeviceCodeSmp as u8
            } else {
                HidCommand::GetDeviceCodeSm3 as u8
            };
            
            let mut cmd2 = [0u8; 10];
            cmd2[0] = 0;
            cmd2[1] = code_cmd;
            
            if device.write(&cmd2).is_err() {
                return None;
            }
            
            match device.read_timeout(&mut buf, 500) {
                Ok(n) if n > 2 => {
                    // Device code is ASCII string starting at offset 1
                    let code_bytes: Vec<u8> = buf[1..n].iter()
                        .take_while(|&&b| b != 0 && b.is_ascii())
                        .copied()
                        .collect();
                    if !code_bytes.is_empty() {
                        let code = String::from_utf8_lossy(&code_bytes).to_string();
                        info!("[SpeechMike] Device code: {}", code);
                        return Some(code);
                    }
                }
                _ => {}
            }
        }
        _ => {
            // Try SpeechOne command
            let mut cmd_so = [0u8; 10];
            cmd_so[0] = 0;
            cmd_so[1] = HidCommand::GetDeviceCodeSo as u8;
            
            if device.write(&cmd_so).is_ok() {
                if let Ok(n) = device.read_timeout(&mut buf, 500) {
                    if n > 2 {
                        let code_bytes: Vec<u8> = buf[1..n].iter()
                            .take_while(|&&b| b != 0 && b.is_ascii())
                            .copied()
                            .collect();
                        if !code_bytes.is_empty() {
                            let code = String::from_utf8_lossy(&code_bytes).to_string();
                            info!("[SpeechMike] Device code (SO): {}", code);
                            return Some(code);
                        }
                    }
                }
            }
        }
    }
    
    None
}

/// Phase 2: Read current event mode from device
fn get_event_mode(device: &hidapi::HidDevice) -> Option<EventMode> {
    let mut cmd = [0u8; 10];
    cmd[0] = 0;
    cmd[1] = HidCommand::GetEventMode as u8;
    
    if device.write(&cmd).is_err() {
        return None;
    }
    
    let mut buf = [0u8; 64];
    match device.read_timeout(&mut buf, 500) {
        Ok(n) if n > 1 && buf[0] == HidCommand::GetEventMode as u8 => {
            match buf[1] {
                0 => Some(EventMode::Hid),
                1 => Some(EventMode::Keyboard),
                2 => Some(EventMode::Browser),
                3 => Some(EventMode::DragonForWindows),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Phase 2: Force device into HID event mode
fn set_event_mode_hid(device: &hidapi::HidDevice) -> bool {
    let mut cmd = [0u8; 10];
    cmd[0] = 0;
    cmd[1] = HidCommand::SetEventMode as u8;
    cmd[2] = EventMode::Hid as u8;
    
    match device.write(&cmd) {
        Ok(_) => {
            info!("[SpeechMike] Event mode switched to HID");
            true
        }
        Err(e) => {
            warn!("[SpeechMike] Failed to set event mode: {}", e);
            false
        }
    }
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
    let led_tx_store = state.led_tx.clone();
    
    // Don't start if already running
    if running.load(Ordering::SeqCst) {
        warn!("[SpeechMike] Thread déjà en cours d'exécution");
        return;
    }
    
    running.store(true, Ordering::SeqCst);
    
    thread::spawn(move || {
        info!("[SpeechMike] Thread de détection HID démarré");
        
        let mut last_bitmask: u32 = 0;
        let mut last_action_time = std::time::Instant::now();
        let debounce_ms = 150u128;
        
        loop {
            if !running.load(Ordering::SeqCst) {
                info!("[SpeechMike] Thread arrêté (signal reçu)");
                break;
            }
            
            let api = match HidApi::new() {
                Ok(api) => api,
                Err(e) => {
                    debug!("[SpeechMike] hidapi init error (retrying in 5s): {}", e);
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };
            
            // Phase 2: Scan for supported devices WITH usage_page filtering
            let mut found_device: Option<(u16, u16, &str, String)> = None;
            for device_info in api.device_list() {
                let vid = device_info.vendor_id();
                let pid = device_info.product_id();
                
                if let Some(filter) = is_supported_device(vid, pid) {
                    // Phase 2: Filter by usage_page to pick the correct HID interface
                    let dev_usage_page = device_info.usage_page();
                    let dev_usage = device_info.usage();
                    
                    if dev_usage_page != filter.usage_page || dev_usage != filter.usage {
                        debug!("[SpeechMike] Skipping interface usagePage:{:#06x} usage:{} (expected {:#06x}/{})", 
                            dev_usage_page, dev_usage, filter.usage_page, filter.usage);
                        continue;
                    }
                    
                    let path = device_info.path().to_string_lossy().to_string();
                    info!("[SpeechMike] ✅ Périphérique détecté: {} (VID:{:04x} PID:{:04x} usagePage:{:#06x})", 
                        filter.description, vid, pid, dev_usage_page);
                    found_device = Some((vid, pid, filter.description, path));
                    break;
                }
            }
            
            let (vid, pid, desc, path) = match found_device {
                Some(d) => d,
                None => {
                    if let Ok(mut s) = status.lock() {
                        if s.connected {
                            s.connected = false;
                            s.device_name = "Aucun périphérique détecté".to_string();
                            s.vendor_id = 0;
                            s.product_id = 0;
                            s.device_code = None;
                            s.has_slider = false;
                            s.event_mode = None;
                            let _ = app_handle.emit_all("airadcr:speechmike_disconnected", ());
                            info!("[SpeechMike] Périphérique déconnecté");
                        }
                    }
                    // Clear LED channel
                    if let Ok(mut lt) = led_tx_store.lock() {
                        *lt = None;
                    }
                    debug!("[SpeechMike] Aucun périphérique trouvé, nouvelle tentative dans 3s");
                    thread::sleep(Duration::from_secs(3));
                    continue;
                }
            };
            
            // Phase 2: Open by path (not VID/PID) to target the correct interface
            let device = match api.open_path(&std::ffi::CString::new(path.clone()).unwrap_or_default()) {
                Ok(d) => d,
                Err(_) => {
                    // Fallback to VID/PID open
                    match api.open(vid, pid) {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("[SpeechMike] ⚠️ Impossible d'ouvrir le périphérique: {}", e);
                            if let Ok(mut s) = status.lock() {
                                s.connected = false;
                                s.device_name = format!("{} (verrouillé)", desc);
                            }
                            thread::sleep(Duration::from_secs(10));
                            continue;
                        }
                    }
                }
            };
            
            // Create LED command channel for this device session
            let (led_sender, led_receiver) = std::sync::mpsc::channel::<SimpleLedState>();
            if let Ok(mut lt) = led_tx_store.lock() {
                *lt = Some(led_sender);
            }
            
            // Phase 2: Fetch device code to identify slider models
            let device_code = fetch_device_code(&device);
            let has_slider = device_code.as_ref().map_or(false, |c| is_slider_model(c));
            
            // Phase 2: Check and switch event mode to HID
            let event_mode = get_event_mode(&device);
            let mode_str = match event_mode {
                Some(EventMode::Hid) => "HID",
                Some(EventMode::Keyboard) => "Keyboard",
                Some(EventMode::Browser) => "Browser",
                Some(EventMode::DragonForWindows) => "DragonForWindows",
                None => "Unknown",
            };
            
            if event_mode.is_some() && event_mode != Some(EventMode::Hid) {
                warn!("[SpeechMike] ⚠️ Mode actuel: {} — passage en mode HID", mode_str);
                set_event_mode_hid(&device);
                thread::sleep(Duration::from_millis(200));
            }
            
            // Update status
            if let Ok(mut s) = status.lock() {
                s.connected = true;
                s.device_name = desc.to_string();
                s.vendor_id = vid;
                s.product_id = pid;
                s.device_code = device_code.clone();
                s.has_slider = has_slider;
                s.event_mode = Some(mode_str.to_string());
            }
            
            let connect_status = SpeechMikeStatus {
                connected: true,
                device_name: desc.to_string(),
                vendor_id: vid,
                product_id: pid,
                device_code: device_code.clone(),
                has_slider,
                event_mode: Some(mode_str.to_string()),
            };
            let _ = app_handle.emit_all("airadcr:speechmike_connected", &connect_status);
            info!("[SpeechMike] 🎤 Connecté: {} (natif HID) code={:?} slider={}", desc, device_code, has_slider);
            
            // Set LED to idle (green)
            if let Err(e) = set_led_state(&device, SimpleLedState::Off) {
                debug!("[SpeechMike] LED control non supporté: {}", e);
            }
            
            let is_pm4 = is_powermic4(vid, pid);
            device.set_blocking_mode(false).ok();
            
            // Main polling loop
            let mut buf = [0u8; 64];
            loop {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                
                match device.read_timeout(&mut buf, 10) {
                    Ok(0) => {
                        // No button data — check for pending LED commands
                        while let Ok(led_state) = led_receiver.try_recv() {
                            debug!("[SpeechMike] 💡 LED command received: {:?}", led_state);
                            if let Err(e) = set_led_state(&device, led_state) {
                                warn!("[SpeechMike] LED write failed: {}", e);
                            }
                        }
                        continue;
                    }
                    Ok(n) => {
                        // Also process LED commands when we have button data
                        while let Ok(led_state) = led_receiver.try_recv() {
                            debug!("[SpeechMike] 💡 LED command received: {:?}", led_state);
                            if let Err(e) = set_led_state(&device, led_state) {
                                warn!("[SpeechMike] LED write failed: {}", e);
                            }
                        }
                        let bitmask = decode_input_report(&buf[..n], is_pm4, has_slider);
                        
                        if bitmask != last_bitmask {
                            let previously_pressed = last_bitmask;
                            last_bitmask = bitmask;
                            
                            let newly_pressed = bitmask & !previously_pressed;
                            
                            if newly_pressed != 0 {
                                let buttons = extract_buttons(newly_pressed);
                                let now = std::time::Instant::now();
                                
                                for button in buttons {
                                    if now.duration_since(last_action_time).as_millis() < debounce_ms {
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
                        warn!("[SpeechMike] ❌ Erreur lecture HID: {} - reconnexion...", e);
                        last_bitmask = 0;
                        
                        if let Ok(mut s) = status.lock() {
                            s.connected = false;
                            s.device_name = "Déconnecté".to_string();
                        }
                        if let Ok(mut lt) = led_tx_store.lock() {
                            *lt = None;
                        }
                        let _ = app_handle.emit_all("airadcr:speechmike_disconnected", ());
                        
                        break;
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
