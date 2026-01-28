//! Option Function Select (OFS) registers configuration
//!
//! RA MCUs have flash-based option registers that configure various system parameters
//! that take effect immediately after reset, before any code executes:
//!
//! - **OFS0**: IWDT (Independent Watchdog), WDT (Watchdog), voltage detection, security
//! - **OFS1**: HOCO frequency selection, additional settings
//! - **OSIS/ID_CODE**: Flash security/ID code
//! - **SECMPU**: Security MPU settings (on some devices)
//!
//! These values are placed in flash at fixed addresses via linker sections.
//! The linker script must define the corresponding sections:
//! - `.option_setting_ofs0`
//! - `.option_setting_ofs1`
//! - `.option_setting_osis`
//! - `.option_setting_secmpu` (if security MPU is available)
//!
//! # Example
//!
//! ```rust,ignore
//! use embassy_ra::system::option_bytes::{OptionBytes, Ofs0, Ofs1, WdtConfig};
//!
//! // Define option bytes with WDT disabled (default)
//! #[link_section = ".option_setting_ofs0"]
//! #[used]
//! static OFS0: u32 = Ofs0::default().to_u32();
//!
//! #[link_section = ".option_setting_ofs1"]  
//! #[used]
//! static OFS1: u32 = Ofs1::default().to_u32();
//! ```

// ============================================================================
// OFS0 - Option Function Select Register 0
// ============================================================================
// Address: 0x0000_0400 (RA2) or varies by family
// Contains: IWDT, WDT, voltage detection, security settings

/// Independent Watchdog Timer (IWDT) timeout period selection
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum IwdtTimeout {
    /// 128 cycles
    Cycles128 = 0b00,
    /// 512 cycles
    Cycles512 = 0b01,
    /// 1024 cycles
    Cycles1024 = 0b10,
    /// 2048 cycles (default - longest timeout)
    #[default]
    Cycles2048 = 0b11,
}

/// IWDT dedicated clock divider
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum IwdtClockDiv {
    /// Divide by 1
    Div1 = 0b0000,
    /// Divide by 16
    Div16 = 0b0010,
    /// Divide by 32
    Div32 = 0b0011,
    /// Divide by 64
    Div64 = 0b0100,
    /// Divide by 128 (default)
    #[default]
    Div128 = 0b1111,
    /// Divide by 256
    Div256 = 0b0101,
}

/// IWDT window end position
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum IwdtWindowEnd {
    /// 75% of timeout
    Percent75 = 0b00,
    /// 50% of timeout
    Percent50 = 0b01,
    /// 25% of timeout
    Percent25 = 0b10,
    /// 0% - no window (default)
    #[default]
    Percent0 = 0b11,
}

/// IWDT window start position
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum IwdtWindowStart {
    /// 25% of timeout
    Percent25 = 0b00,
    /// 50% of timeout
    Percent50 = 0b01,
    /// 75% of timeout
    Percent75 = 0b10,
    /// 100% - no window (default)
    #[default]
    Percent100 = 0b11,
}

/// Watchdog Timer (WDT) start mode
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WdtStartMode {
    /// WDT is stopped after reset (can be started by register)
    #[default]
    RegisterStart,
    /// WDT starts automatically after reset
    AutoStart,
}

/// Watchdog Timer (WDT) timeout period
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum WdtTimeout {
    /// 1024 cycles
    Cycles1024 = 0b00,
    /// 4096 cycles
    Cycles4096 = 0b01,
    /// 8192 cycles
    Cycles8192 = 0b10,
    /// 16384 cycles (default - longest timeout)
    #[default]
    Cycles16384 = 0b11,
}

/// WDT clock divider
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum WdtClockDiv {
    /// Divide by 4
    Div4 = 0b0001,
    /// Divide by 64
    Div64 = 0b0100,
    /// Divide by 128 (default)
    #[default]
    Div128 = 0b1111,
    /// Divide by 512
    Div512 = 0b0110,
    /// Divide by 2048
    Div2048 = 0b0111,
    /// Divide by 8192
    Div8192 = 0b1000,
}

/// Reset control - what happens on WDT/IWDT underflow
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WdtResetControl {
    /// Generate NMI on underflow
    Nmi,
    /// Generate reset on underflow (default)
    #[default]
    Reset,
}

/// IWDT configuration for OFS0
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IwdtConfig {
    /// Start mode: true = auto-start after reset
    pub auto_start: bool,
    /// Timeout period
    pub timeout: IwdtTimeout,
    /// Clock divider
    pub clock_div: IwdtClockDiv,
    /// Window end position
    pub window_end: IwdtWindowEnd,
    /// Window start position
    pub window_start: IwdtWindowStart,
    /// Reset control
    pub reset_control: WdtResetControl,
    /// Stop counting in sleep mode
    pub stop_in_sleep: bool,
}

/// WDT configuration for OFS0
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WdtConfig {
    /// Start mode
    pub start_mode: WdtStartMode,
    /// Timeout period
    pub timeout: WdtTimeout,
    /// Clock divider
    pub clock_div: WdtClockDiv,
    /// Window end position (same type as IWDT)
    pub window_end: IwdtWindowEnd,
    /// Window start position (same type as IWDT)
    pub window_start: IwdtWindowStart,
    /// Reset control
    pub reset_control: WdtResetControl,
    /// Stop counting in sleep mode
    pub stop_in_sleep: bool,
}

/// Voltage detection configuration
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VoltageDetectConfig {
    /// Enable voltage monitoring 1 reset
    pub lvd1_reset_enable: bool,
    /// Enable voltage monitoring 2 reset  
    pub lvd2_reset_enable: bool,
}

/// OFS0 register configuration
///
/// This register controls IWDT, WDT, and voltage detection settings.
/// The default value (all 0xFF) disables both watchdogs.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ofs0 {
    /// IWDT configuration (None = IWDT disabled)
    pub iwdt: Option<IwdtConfig>,
    /// WDT configuration (None = WDT disabled/register-start mode)
    pub wdt: Option<WdtConfig>,
    /// Voltage detection configuration
    pub voltage_detect: VoltageDetectConfig,
}

impl Default for Ofs0 {
    fn default() -> Self {
        Self {
            iwdt: None,  // IWDT disabled
            wdt: None,   // WDT in register-start mode (not auto-start)
            voltage_detect: VoltageDetectConfig::default(),
        }
    }
}

impl Ofs0 {
    /// Convert configuration to raw register value
    ///
    /// OFS0 bit layout (varies slightly by device, this is typical RA2/RA4/RA6):
    /// - Bits [1:0]: IWDT timeout period (IWDTTOPS)
    /// - Bits [5:2]: IWDT clock divider (IWDTCKS)
    /// - Bits [7:6]: IWDT window end (IWDTRPES)
    /// - Bits [9:8]: IWDT window start (IWDTRPSS)
    /// - Bit [10]: IWDT reset/NMI select (IWDTRSTIRQS)
    /// - Bit [12]: IWDT stop in sleep (IWDTSTRT)
    /// - Bit [14]: IWDT start mode (IWDTSLCSTP)
    /// - Bit [16]: WDT start mode (WDTSTRT)
    /// - Bits [18:17]: WDT timeout (WDTTOPS)
    /// - Bits [23:20]: WDT clock divider (WDTCKS)  
    /// - Bits [25:24]: WDT window end (WDTRPES)
    /// - Bits [27:26]: WDT window start (WDTRPSS)
    /// - Bit [28]: WDT reset/NMI select (WDTRSTIRQS)
    /// - Bit [30]: WDT stop in sleep
    pub const fn to_u32(&self) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF; // Default: all bits set (disabled)
        
        // IWDT configuration
        if let Some(ref iwdt) = self.iwdt {
            // Clear and set IWDT bits
            val &= !(0b11 << 0);  // Clear timeout bits
            val |= (iwdt.timeout as u32) << 0;
            
            val &= !(0b1111 << 2);  // Clear clock div bits
            val |= (iwdt.clock_div as u32) << 2;
            
            val &= !(0b11 << 6);  // Clear window end bits
            val |= (iwdt.window_end as u32) << 6;
            
            val &= !(0b11 << 8);  // Clear window start bits
            val |= (iwdt.window_start as u32) << 8;
            
            // Reset/NMI select (bit 10): 0 = NMI, 1 = Reset
            if matches!(iwdt.reset_control, WdtResetControl::Nmi) {
                val &= !(1 << 10);
            }
            
            // Stop in sleep (bit 12): 0 = count, 1 = stop
            if iwdt.stop_in_sleep {
                val |= 1 << 12;
            } else {
                val &= !(1 << 12);
            }
            
            // Start mode (bit 14): 0 = auto-start, 1 = stop
            if iwdt.auto_start {
                val &= !(1 << 14);
            }
        }
        
        // WDT configuration
        if let Some(ref wdt) = self.wdt {
            // Start mode (bit 16): 0 = auto-start, 1 = register-start
            match wdt.start_mode {
                WdtStartMode::AutoStart => val &= !(1 << 16),
                WdtStartMode::RegisterStart => val |= 1 << 16,
            }
            
            val &= !(0b11 << 17);  // Clear timeout bits
            val |= (wdt.timeout as u32) << 17;
            
            val &= !(0b1111 << 20);  // Clear clock div bits
            val |= (wdt.clock_div as u32) << 20;
            
            val &= !(0b11 << 24);  // Clear window end bits
            val |= (wdt.window_end as u32) << 24;
            
            val &= !(0b11 << 26);  // Clear window start bits
            val |= (wdt.window_start as u32) << 26;
            
            // Reset/NMI select (bit 28): 0 = NMI, 1 = Reset
            if matches!(wdt.reset_control, WdtResetControl::Nmi) {
                val &= !(1 << 28);
            }
            
            // Stop in sleep (bit 30): 0 = count, 1 = stop
            if wdt.stop_in_sleep {
                val |= 1 << 30;
            } else {
                val &= !(1 << 30);
            }
        }
        
        val
    }
    
    /// Create OFS0 with both watchdogs disabled (default safe value)
    pub const fn disabled() -> Self {
        Self {
            iwdt: None,
            wdt: None,
            voltage_detect: VoltageDetectConfig {
                lvd1_reset_enable: false,
                lvd2_reset_enable: false,
            },
        }
    }
}

// ============================================================================
// OFS1 - Option Function Select Register 1
// ============================================================================
// Contains: HOCO frequency, additional settings
//
// IMPORTANT: The HOCOFRQ bit field offset differs between families:
// - RA2 family: Offset 12, Mask 0xFFFF8FFF (bits [14:12])
// - RA6/RA4 family (M33 with TZ): Offset 9, Mask 0xFFFFF9FF (bits [10:9])
// - RA8 family: Offset 9, Mask 0xFFFFF1FF (bits [11:9])

/// HOCO frequency selection for OFS1
///
/// Note: Not all frequencies are available on all devices.
/// Check the device datasheet for supported values.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Ofs1HocoFreq {
    /// 24 MHz
    Mhz24 = 0b000,
    /// 32 MHz (RA2 only)
    Mhz32 = 0b010,
    /// 48 MHz (default)
    #[default]
    Mhz48 = 0b100,
    /// 64 MHz
    Mhz64 = 0b101,
}

/// OFS1 register configuration
///
/// This register controls HOCO frequency and other settings.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ofs1 {
    /// HOCO frequency selection
    pub hoco_freq: Ofs1HocoFreq,
}

impl Default for Ofs1 {
    fn default() -> Self {
        Self {
            hoco_freq: Ofs1HocoFreq::Mhz48,
        }
    }
}

impl Ofs1 {
    /// Convert configuration to raw register value for RA2 family
    /// 
    /// RA2 OFS1 bit layout:
    /// - Bits [14:12]: HOCOFRQ - HOCO frequency selection
    /// - Mask: 0xFFFF8FFF, Offset: 12
    #[cfg(any(ra2e1, ra2e2, ra2l1))]
    pub const fn to_u32(&self) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF;
        val &= 0xFFFF8FFF;  // Clear bits [14:12]
        val |= (self.hoco_freq as u32) << 12;
        val
    }

    /// Convert configuration to raw register value for RA6/RA4 family (with TrustZone)
    /// 
    /// RA6E2/RA6M4/RA4E2 OFS1 bit layout:
    /// - Bits [10:9]: HOCOFRQ - HOCO frequency selection
    /// - Mask: 0xFFFFF9FF, Offset: 9
    #[cfg(any(ra6e2, ra6m4, ra4e2, ra4m2, ra4m3))]
    pub const fn to_u32(&self) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF;
        val &= 0xFFFFF9FF;  // Clear bits [10:9]
        val |= (self.hoco_freq as u32) << 9;
        val
    }

    /// Convert configuration to raw register value for RA6 family (without TrustZone)
    /// 
    /// RA6M3/RA6M1/RA6M2 OFS1 bit layout:
    /// - Bits [10:9]: HOCOFRQ - HOCO frequency selection  
    /// - Mask: 0xFFFFF9FF, Offset: 9
    #[cfg(any(ra6m1, ra6m2, ra6m3))]
    pub const fn to_u32(&self) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF;
        val &= 0xFFFFF9FF;  // Clear bits [10:9]
        val |= (self.hoco_freq as u32) << 9;
        val
    }

    /// Convert configuration to raw register value for RA8 family
    /// 
    /// RA8M1/RA8D1 OFS1 bit layout:
    /// - Bits [11:9]: HOCOFRQ - HOCO frequency selection
    /// - Mask: 0xFFFFF1FF, Offset: 9
    #[cfg(any(ra8m1, ra8d1, ra8e1, ra8t1))]
    pub const fn to_u32(&self) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF;
        val &= 0xFFFFF1FF;  // Clear bits [11:9]
        val |= (self.hoco_freq as u32) << 9;
        val
    }

    /// Fallback for unsupported families - uses RA2 layout
    #[cfg(not(any(
        ra2e1, ra2e2, ra2l1,
        ra6e2, ra6m4, ra4e2, ra4m2, ra4m3,
        ra6m1, ra6m2, ra6m3,
        ra8m1, ra8d1, ra8e1, ra8t1
    )))]
    pub const fn to_u32(&self) -> u32 {
        let mut val: u32 = 0xFFFF_FFFF;
        val &= 0xFFFF8FFF;  // Default to RA2 layout: bits [14:12]
        val |= (self.hoco_freq as u32) << 12;
        val
    }
}

// ============================================================================
// ID Code (OSIS) - Flash Security
// ============================================================================

/// ID Code for flash security
///
/// The ID code protects the flash from unauthorized access.
/// - All 0xFF = unlocked (no protection)
/// - Any other value = locked (requires matching ID to access)
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdCode {
    /// 16-byte ID code (4 x 32-bit words)
    /// Note: On some devices there may be padding between words
    pub code: [u32; 4],
}

impl Default for IdCode {
    fn default() -> Self {
        Self {
            // All 0xFF = unlocked
            code: [0xFFFF_FFFF; 4],
        }
    }
}

impl IdCode {
    /// Create an unlocked (unprotected) ID code
    pub const fn unlocked() -> Self {
        Self {
            code: [0xFFFF_FFFF; 4],
        }
    }
    
    /// Create a locked ID code with the specified value
    /// 
    /// Warning: Once programmed with a non-0xFF value, the flash is protected
    /// and requires this ID to access.
    pub const fn locked(code: [u32; 4]) -> Self {
        Self { code }
    }
    
    /// Convert to raw array for linker section
    /// 
    /// On devices with OSIS padding (1 word between each ID word),
    /// use `to_padded_array()` instead.
    pub const fn to_array(&self) -> [u32; 4] {
        self.code
    }
    
    /// Convert to padded array for devices that require padding between ID words
    ///
    /// Some RA devices have 32-bit padding between each ID word in the OSIS area.
    /// Check BSP_FEATURE_BSP_OSIS_PADDING for your device.
    pub const fn to_padded_array(&self) -> [u32; 8] {
        [
            self.code[0],
            0xFFFF_FFFF, // Padding
            self.code[1],
            0xFFFF_FFFF, // Padding
            self.code[2],
            0xFFFF_FFFF, // Padding
            self.code[3],
            0xFFFF_FFFF, // Padding
        ]
    }
}

// ============================================================================
// Security MPU Settings
// ============================================================================

/// Security MPU configuration
///
/// Controls access permissions for code flash regions.
/// Only available on devices with BSP_FEATURE_BSP_HAS_SECURITY_MPU = 1
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SecMpu {
    /// Raw SECMPU register values
    pub values: [u32; 4],
}

impl Default for SecMpu {
    fn default() -> Self {
        Self {
            // All 0xFF = no restrictions
            values: [0xFFFF_FFFF; 4],
        }
    }
}

impl SecMpu {
    /// Create with no security restrictions (default)
    pub const fn unrestricted() -> Self {
        Self {
            values: [0xFFFF_FFFF; 4],
        }
    }
}

// ============================================================================
// Combined Option Bytes Structure
// ============================================================================

/// Complete option bytes configuration
///
/// This struct provides a convenient way to configure all option bytes together.
/// Individual fields can be placed in linker sections as needed.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OptionBytes {
    /// OFS0 - IWDT, WDT, voltage detection
    pub ofs0: Ofs0,
    /// OFS1 - HOCO frequency
    pub ofs1: Ofs1,
    /// ID Code for flash security
    pub id_code: IdCode,
    /// Security MPU settings
    pub sec_mpu: SecMpu,
}

impl OptionBytes {
    /// Create default option bytes (all features disabled/unlocked)
    pub const fn new() -> Self {
        Self {
            ofs0: Ofs0 {
                iwdt: None,
                wdt: None,
                voltage_detect: VoltageDetectConfig {
                    lvd1_reset_enable: false,
                    lvd2_reset_enable: false,
                },
            },
            ofs1: Ofs1 {
                hoco_freq: Ofs1HocoFreq::Mhz48,
            },
            id_code: IdCode::unlocked(),
            sec_mpu: SecMpu::unrestricted(),
        }
    }
}

// ============================================================================
// Default Option Bytes - Automatically provided by embassy-ra
// ============================================================================
//
// These defaults are provided so users don't need to define option bytes
// unless they want to customize them. Users can override by defining their
// own statics with the same link_section in their application.
//
// To override, define in your main.rs:
// ```
// #[link_section = ".option_setting_ofs0"]
// #[used]
// pub static OFS0: u32 = my_custom_ofs0_value;
// ```

// ----------------------------------------------------------------------------
// RA2 Family Defaults (RA2E1, RA2E2, RA2L1)
// ----------------------------------------------------------------------------

/// Default OFS0 for RA2 family: Both watchdogs disabled
#[cfg(any(ra2e1, ra2e2, ra2l1))]
#[link_section = ".option_setting_ofs0"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS0: u32 = Ofs0::disabled().to_u32();

/// Default OFS1 for RA2 family: 48 MHz HOCO
#[cfg(any(ra2e1, ra2e2, ra2l1))]
#[link_section = ".option_setting_ofs1"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS1: u32 = Ofs1 { hoco_freq: Ofs1HocoFreq::Mhz48 }.to_u32();

/// Default ID Code (OSIS) for RA2 family: Unlocked with padding
#[cfg(any(ra2e1, ra2e2, ra2l1))]
#[link_section = ".option_setting_osis"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OSIS: [u32; 8] = IdCode::unlocked().to_padded_array();

/// Default SECMPU for RA2 family: No restrictions
#[cfg(any(ra2e1, ra2e2, ra2l1))]
#[link_section = ".option_setting_secmpu"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_SECMPU: [u32; 4] = SecMpu::unrestricted().values;

// ----------------------------------------------------------------------------
// RA6 Family Defaults with TrustZone (RA6E2, RA6M4, RA6M5)
// ----------------------------------------------------------------------------

/// Default OFS0 for RA6 TZ family: Both watchdogs disabled
#[cfg(any(ra6e2, ra6m4, ra6m5))]
#[link_section = ".option_setting_ofs0"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS0: u32 = Ofs0::disabled().to_u32();

/// Default OFS1_SEC for RA6 TZ family: 48 MHz HOCO
/// Note: RA6 with TZ uses OFS1_SEC section
#[cfg(any(ra6e2, ra6m4, ra6m5))]
#[link_section = ".option_setting_ofs1_sec"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS1_SEC: u32 = Ofs1 { hoco_freq: Ofs1HocoFreq::Mhz48 }.to_u32();

/// Default BPS_SEC for RA6 TZ family: No block protection
#[cfg(any(ra6e2, ra6m4, ra6m5))]
#[link_section = ".option_setting_bps_sec"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_BPS_SEC: u32 = 0xFFFF_FFFF; // No struct yet for BPS

/// Default PBPS_SEC for RA6 TZ family: No block protection
#[cfg(any(ra6e2, ra6m4, ra6m5))]
#[link_section = ".option_setting_pbps_sec"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_PBPS_SEC: u32 = 0xFFFF_FFFF; // No struct yet for PBPS

/// Default OSIS for RA6 TZ family: Unlocked
#[cfg(any(ra6e2, ra6m4, ra6m5))]
#[link_section = ".option_setting_osis"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OSIS: [u32; 4] = IdCode::unlocked().to_array();

// ----------------------------------------------------------------------------
// RA6 Family Defaults without TrustZone (RA6M1, RA6M2, RA6M3)
// ----------------------------------------------------------------------------

/// Default OFS0 for RA6 non-TZ family: Both watchdogs disabled
#[cfg(any(ra6m1, ra6m2, ra6m3))]
#[link_section = ".option_setting_ofs0"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS0: u32 = Ofs0::disabled().to_u32();

/// Default OFS1 for RA6 non-TZ family: 48 MHz HOCO
#[cfg(any(ra6m1, ra6m2, ra6m3))]
#[link_section = ".option_setting_ofs1"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS1: u32 = Ofs1 { hoco_freq: Ofs1HocoFreq::Mhz48 }.to_u32();

// ----------------------------------------------------------------------------
// RA4 Family Defaults (RA4M1, RA4E1, RA4W1 - no TrustZone)
// ----------------------------------------------------------------------------

/// Default OFS0 for RA4 non-TZ family
#[cfg(any(ra4m1, ra4e1, ra4w1))]
#[link_section = ".option_setting_ofs0"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS0: u32 = Ofs0::disabled().to_u32();

/// Default OFS1 for RA4 non-TZ family
#[cfg(any(ra4m1, ra4e1, ra4w1))]
#[link_section = ".option_setting_ofs1"]
#[used]
#[no_mangle]
pub static __EMBASSY_RA_OFS1: u32 = Ofs1 { hoco_freq: Ofs1HocoFreq::Mhz48 }.to_u32();
