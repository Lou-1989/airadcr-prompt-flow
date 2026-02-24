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
}

/// All supported SpeechMike and compatible devices
pub const SUPPORTED_DEVICES: &[HidDeviceFilter] = &[
    // Wired SpeechMikes (LFH35xx, LFH36xx, SMP37xx, SMP38xx) in HID mode
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0c1c, description: "SpeechMike Premium (LFH35xx/36xx/SMP37xx/38xx)" },
    // SpeechMike Premium Air (SMP40xx) in HID mode
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0c1d, description: "SpeechMike Premium Air (SMP40xx)" },
    // SpeechOne (PSM6000) or SpeechMike Ambient (PSM5000)
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0c1e, description: "SpeechOne (PSM6000) / Ambient (PSM5000)" },
    // All SpeechMikes in Browser/Gamepad mode
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x0fa0, description: "SpeechMike (Browser/Gamepad mode)" },
    // PowerMic IV
    HidDeviceFilter { vendor_id: 0x0554, product_id: 0x0064, description: "Nuance PowerMic IV" },
    // PowerMic III
    HidDeviceFilter { vendor_id: 0x0554, product_id: 0x1001, description: "Nuance PowerMic III" },
    // Foot Controls
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x1844, description: "Philips Foot Control ACC2310/2320" },
    HidDeviceFilter { vendor_id: 0x0911, product_id: 0x091a, description: "Philips Foot Control ACC2330" },
];

/// Button events matching the SDK's ButtonEvent enum bitmask values
/// These are the OUTPUT bitmask values (after mapping from input report)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ButtonEvent {
    None      = 0,
    Rewind    = 1 << 0,   // 0x0001
    Play      = 1 << 1,   // 0x0002
    Forward   = 1 << 2,   // 0x0004
    InsOvr    = 1 << 4,   // 0x0010
    Record    = 1 << 5,   // 0x0020
    Command   = 1 << 6,   // 0x0040
    Stop      = 1 << 8,   // 0x0100
    Instr     = 1 << 9,   // 0x0200
    F1A       = 1 << 10,  // 0x0400
    F2B       = 1 << 11,  // 0x0800
    F3C       = 1 << 12,  // 0x1000
    F4D       = 1 << 13,  // 0x2000
    EolPrio   = 1 << 14,  // 0x4000
    Transcribe = 1 << 15, // 0x8000
}

/// Button mapping from HID input report bitmask to ButtonEvent
/// This maps the RAW input report bits to our ButtonEvent enum
/// From speechmike_hid_device.ts BUTTON_MAPPINGS_SPEECHMIKE
/// Key: ButtonEvent, Value: input report bitmask
pub const BUTTON_MAPPINGS_SPEECHMIKE: &[(ButtonEvent, u16)] = &[
    (ButtonEvent::Rewind,    1 << 12),
    (ButtonEvent::Play,      1 << 10),
    (ButtonEvent::Forward,   1 << 11),
    (ButtonEvent::InsOvr,    1 << 14),
    (ButtonEvent::Record,    1 << 8),
    (ButtonEvent::Command,   1 << 5),
    (ButtonEvent::Stop,      1 << 9),
    (ButtonEvent::Instr,     1 << 15),
    (ButtonEvent::F1A,       1 << 1),
    (ButtonEvent::F2B,       1 << 2),
    (ButtonEvent::F3C,       1 << 3),
    (ButtonEvent::F4D,       1 << 4),
    (ButtonEvent::EolPrio,   1 << 13),
];

/// Button mapping for PowerMic IV (different layout)
pub const BUTTON_MAPPINGS_POWERMIC4: &[(ButtonEvent, u16)] = &[
    (ButtonEvent::Rewind,    1 << 13),   // TAB_BACKWARD mapped to Rewind
    (ButtonEvent::Play,      1 << 10),
    (ButtonEvent::Forward,   1 << 14),
    (ButtonEvent::Record,    1 << 8),
    (ButtonEvent::Command,   1 << 5),
    (ButtonEvent::F1A,       1 << 1),
    (ButtonEvent::F2B,       1 << 2),
    (ButtonEvent::F3C,       1 << 3),
    (ButtonEvent::F4D,       1 << 4),
    (ButtonEvent::Transcribe, 1 << 15),  // ENTER_SELECT
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
            // Record LED Red ON
            report[5] |= (LedMode::On as u8) << 2;   // RECORD_LED_RED bits 2-3
        }
        SimpleLedState::RecordStandbyInsert => {
            // Record LED Green BLINK + INS_OWR Green BLINK
            report[5] |= LedMode::BlinkSlow as u8;
            report[6] |= (LedMode::BlinkSlow as u8) << 4;
        }
        SimpleLedState::RecordStandbyOverwrite => {
            // Record LED Red BLINK
            report[5] |= (LedMode::BlinkSlow as u8) << 2;
        }
    }
    
    report
}

/// Check if a device matches any supported SpeechMike filter
pub fn is_supported_device(vendor_id: u16, product_id: u16) -> Option<&'static HidDeviceFilter> {
    SUPPORTED_DEVICES.iter().find(|d| d.vendor_id == vendor_id && d.product_id == product_id)
}

/// Check if device is a PowerMic IV (different button mapping)
pub fn is_powermic4(vendor_id: u16, product_id: u16) -> bool {
    vendor_id == 0x0554 && product_id == 0x0064
}
