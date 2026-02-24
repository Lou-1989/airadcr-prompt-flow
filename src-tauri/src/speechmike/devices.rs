// 🎤 SpeechMike Device Definitions
// Extracted from Google ChromeLabs dictation_support SDK
// https://github.com/GoogleChromeLabs/dictation_support

/// Supported HID device filters (VendorID, ProductID)
/// Based on dictation_device_manager.ts DEVICE_FILTERS
#[derive(Debug, Clone, Copy)]
pub struct HidDeviceFilter {
    pub vendor_id: u16,
    pub product_id: u16,
    pub description: &'static str,
    /// Expected HID usage_page for this device type
    pub usage_page: u16,
    /// Expected HID usage for this device type
    pub usage: u16,
}

/// All supported SpeechMike and compatible devices
pub const SUPPORTED_DEVICES: &[HidDeviceFilter] = &[
    // Wired SpeechMikes (LFH35xx, LFH36xx, SMP37xx, SMP38xx) in HID mode
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0c1c, description: "SpeechMike Premium (LFH35xx/36xx/SMP37xx/38xx)", usage_page: 0xffa0, usage: 1 },
    // SpeechMike Premium Air (SMP40xx) in HID mode
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0c1d, description: "SpeechMike Premium Air (SMP40xx)", usage_page: 0xffa0, usage: 1 },
    // SpeechOne (PSM6000) or SpeechMike Ambient (PSM5000)
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0c1e, description: "SpeechOne (PSM6000) / Ambient (PSM5000)", usage_page: 0xffa0, usage: 1 },
    // All SpeechMikes in Browser/Gamepad mode
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0fa0, description: "SpeechMike (Browser/Gamepad mode)", usage_page: 1, usage: 4 },
    // PowerMic IV
    HidDeviceFilter { vendor_id: 0x0554, product_id: 0x0064, description: "Nuance PowerMic IV", usage_page: 0xffa0, usage: 1 },
    // PowerMic III
    HidDeviceFilter { vendor_id: 0x0554, product_id: 0x1001, description: "Nuance PowerMic III", usage_page: 0xffa0, usage: 1 },
    // Foot Controls
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x1844, description: "Philips Foot Control ACC2310/2320", usage_page: 0xffa0, usage: 1 },
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x091a, description: "Philips Foot Control ACC2330", usage_page: 0xffa0, usage: 1 },
];

/// Button events matching the SDK's ButtonEvent enum bitmask values
/// These are the OUTPUT bitmask values (after mapping from input report)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ButtonEvent {
    None        = 0,
    ScanEnd     = 1 << 0,   // 0x0001 — barcode scanner end (Phase 3)
    Rewind      = 1 << 1,   // 0x0002 — Note: output bit, not input bit
    Play        = 1 << 2,   // 0x0004
    Forward     = 1 << 3,   // 0x0008
    InsOvr      = 1 << 4,   // 0x0010
    Record      = 1 << 5,   // 0x0020
    Command     = 1 << 6,   // 0x0040
    ScanSuccess = 1 << 7,   // 0x0080 — barcode scanner success (Phase 3)
    Stop        = 1 << 8,   // 0x0100
    Instr       = 1 << 9,   // 0x0200
    F1A         = 1 << 10,  // 0x0400
    F2B         = 1 << 11,  // 0x0800
    F3C         = 1 << 12,  // 0x1000
    F4D         = 1 << 13,  // 0x2000
    EolPrio     = 1 << 14,  // 0x4000
    Transcribe  = 1 << 15,  // 0x8000
    // PowerMic IV extra buttons (no SDK equivalent in SpeechMike)
    TabBackward = 1 << 16,  // PowerMic IV TAB_BACKWARD
    TabForward  = 1 << 17,  // PowerMic IV TAB_FORWARD
}

/// Button mapping from HID input report bitmask to ButtonEvent
/// This maps the RAW input report bits to our ButtonEvent enum
/// From speechmike_hid_device.ts BUTTON_MAPPINGS_SPEECHMIKE
/// Key: ButtonEvent, Value: input report bitmask
pub const BUTTON_MAPPINGS_SPEECHMIKE: &[(ButtonEvent, u16)] = &[
    (ButtonEvent::ScanEnd,   1 << 0),   // Phase 3: barcode scanner
    (ButtonEvent::Rewind,    1 << 12),
    (ButtonEvent::Play,      1 << 10),
    (ButtonEvent::Forward,   1 << 11),
    (ButtonEvent::InsOvr,    1 << 14),
    (ButtonEvent::Record,    1 << 8),
    (ButtonEvent::Command,   1 << 5),
    (ButtonEvent::ScanSuccess, 1 << 7), // Phase 3: barcode scanner
    (ButtonEvent::Stop,      1 << 9),
    (ButtonEvent::Instr,     1 << 15),
    (ButtonEvent::F1A,       1 << 1),
    (ButtonEvent::F2B,       1 << 2),
    (ButtonEvent::F3C,       1 << 3),
    (ButtonEvent::F4D,       1 << 4),
    (ButtonEvent::EolPrio,   1 << 13),
];

/// Button mapping for PowerMic IV (different layout)
/// Fixed: TAB_BACKWARD (1<<12) was missing, REWIND is 1<<13
pub const BUTTON_MAPPINGS_POWERMIC4: &[(ButtonEvent, u16)] = &[
    (ButtonEvent::TabBackward, 1 << 12), // TAB_BACKWARD — was missing!
    (ButtonEvent::Rewind,      1 << 13), // REWIND (correct bit)
    (ButtonEvent::Play,        1 << 10),
    (ButtonEvent::Forward,     1 << 14),
    (ButtonEvent::TabForward,  1 << 11), // TAB_FORWARD — was missing!
    (ButtonEvent::Record,      1 << 8),
    (ButtonEvent::Command,     1 << 5),
    (ButtonEvent::F1A,         1 << 1),
    (ButtonEvent::F2B,         1 << 2),
    (ButtonEvent::F3C,         1 << 3),
    (ButtonEvent::F4D,         1 << 4),
    (ButtonEvent::Transcribe,  1 << 15), // ENTER_SELECT
];

/// HID Commands (from speechmike_hid_device.ts)
#[repr(u8)]
pub enum HidCommand {
    SetLed               = 0x02,
    SetEventMode         = 0x0d,
    ButtonPressEvent     = 0x80,
    IsSpeechMikePremium  = 0x83,
    GetDeviceCodeSm3     = 0x87,
    GetDeviceCodeSmp     = 0x8b,
    GetDeviceCodeSo      = 0x96,
    GetEventMode         = 0x8d,
    WirelessStatusEvent  = 0x94,
    MotionEvent          = 0x9e,
    GetFirmwareVersion   = 0x91,
}

/// Event modes (from speechmike_hid_device.ts)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum EventMode {
    Hid              = 0,
    Keyboard         = 1,
    Browser          = 2,
    DragonForWindows = 3,
}

/// LED indices (from speechmike_hid_device.ts)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LedIndex {
    RecordLedGreen       = 0,
    RecordLedRed         = 1,
    InstructionLedGreen  = 2,
    InstructionLedRed    = 3,
    InsOwrButtonLedGreen = 4,
    InsOwrButtonLedRed   = 5,
    F1ButtonLed          = 6,
    F2ButtonLed          = 7,
    F3ButtonLed          = 8,
    F4ButtonLed          = 9,
}

/// LED mode
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LedMode {
    Off       = 0,
    BlinkSlow = 1,
    BlinkFast = 2,
    On        = 3,
}

/// Predefined LED states
#[derive(Debug, Clone, Copy)]
pub enum SimpleLedState {
    Off,
    RecordInsert,
    RecordOverwrite,
    RecordStandbyInsert,
    RecordStandbyOverwrite,
}

/// Build the LED output report bytes (positions 5, 6, 7) for a simple LED state
/// Phase 3: Now fills all 3 LED bytes completely matching SDK layout:
///   byte[5]: bits 0-1 RECORD_GREEN, 2-3 RECORD_RED, 4-5 INSTR_GREEN, 6-7 INSTR_RED
///   byte[6]: bits 0-3 unused, 4-5 INS_OWR_GREEN, 6-7 INS_OWR_RED
///   byte[7]: bits 0-1 F1, 2-3 F2, 4-5 F3, 6-7 F4
pub fn build_led_report(state: SimpleLedState) -> [u8; 8] {
    let mut report = [0u8; 8];
    
    match state {
        SimpleLedState::Off => {
            // All LEDs off - report stays zeroed
        }
        SimpleLedState::RecordInsert => {
            // Record LED Green ON + INS_OWR Green ON
            report[5] |= LedMode::On as u8;          // RECORD_LED_GREEN bits 0-1
            report[6] |= (LedMode::On as u8) << 4;   // INS_OWR_BUTTON_LED_GREEN bits 4-5
        }
        SimpleLedState::RecordOverwrite => {
            // Record LED Red ON + INS_OWR Red ON
            report[5] |= (LedMode::On as u8) << 2;   // RECORD_LED_RED bits 2-3
            report[6] |= (LedMode::On as u8) << 6;   // INS_OWR_BUTTON_LED_RED bits 6-7
        }
        SimpleLedState::RecordStandbyInsert => {
            // Record LED Green BLINK + INS_OWR Green BLINK
            report[5] |= LedMode::BlinkSlow as u8;
            report[6] |= (LedMode::BlinkSlow as u8) << 4;
        }
        SimpleLedState::RecordStandbyOverwrite => {
            // Record LED Red BLINK + INS_OWR Red BLINK
            report[5] |= (LedMode::BlinkSlow as u8) << 2;
            report[6] |= (LedMode::BlinkSlow as u8) << 6;
        }
    }
    
    report
}

/// Build a granular LED report setting a single LED index to a mode
/// Phase 3: Supports all 10 LED indices
pub fn build_single_led_report(index: LedIndex, mode: LedMode) -> [u8; 8] {
    let mut report = [0u8; 8];
    let mode_val = mode as u8;
    
    match index {
        LedIndex::RecordLedGreen       => report[5] |= mode_val,
        LedIndex::RecordLedRed         => report[5] |= mode_val << 2,
        LedIndex::InstructionLedGreen  => report[5] |= mode_val << 4,
        LedIndex::InstructionLedRed    => report[5] |= mode_val << 6,
        LedIndex::InsOwrButtonLedGreen => report[6] |= mode_val << 4,
        LedIndex::InsOwrButtonLedRed   => report[6] |= mode_val << 6,
        LedIndex::F1ButtonLed          => report[7] |= mode_val,
        LedIndex::F2ButtonLed          => report[7] |= mode_val << 2,
        LedIndex::F3ButtonLed          => report[7] |= mode_val << 4,
        LedIndex::F4ButtonLed          => report[7] |= mode_val << 6,
    }
    
    report
}

/// Slider filter bitmask for models with a slider switch
/// From speechmike_hid_device.ts filterOutputBitMask()
/// These bits correspond to the slider position which is always OR'd into the
/// input report. Without filtering, slider changes trigger false button events.
/// Models with slider: LFH3x10, LFH3x20, SMP3710, SMP3720
pub const SLIDER_FILTER_MASK: u16 = (1 << 5) | (1 << 6); // bits 5 and 6 = slider position

/// Device codes for models with a slider (must be filtered)
/// Obtained via device code fetch sequence
pub const SLIDER_MODEL_DEVICE_CODES: &[&str] = &[
    "LFH3210", "LFH3220", "LFH3510", "LFH3520",
    "SMP3710", "SMP3720",
];

/// Check if a device matches any supported SpeechMike filter
pub fn is_supported_device(vendor_id: u16, product_id: u16) -> Option<&'static HidDeviceFilter> {
    SUPPORTED_DEVICES.iter().find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}

/// Check if device is a PowerMic IV (different button mapping)
pub fn is_powermic4(vendor_id: u16, product_id: u16) -> bool {
    vendor_id == 0x0554 && product_id == 0x0064
}

/// Check if device is in Gamepad/Browser mode (proxy device)
pub fn is_gamepad_mode(product_id: u16) -> bool {
    product_id == 0x0fa0
}

/// Check if a device code string corresponds to a slider model
pub fn is_slider_model(device_code: &str) -> bool {
    SLIDER_MODEL_DEVICE_CODES.iter().any(|&code| device_code.starts_with(code))
}

/// Apply slider filter to raw input bitmask
/// Removes slider position bits that would cause false button triggers
pub fn filter_slider_bits(raw_bitmask: u16) -> u16 {
    raw_bitmask & !SLIDER_FILTER_MASK
}
