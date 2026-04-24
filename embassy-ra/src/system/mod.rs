//! Reset and Clock Control (RCC) for Renesas RA
//!
//! This module handles clock configuration for RA MCUs via the SYSC/SYSTEM peripheral.
//! Different RA families have different clock sources and dividers available.

#![allow(missing_docs)]

use core::mem::MaybeUninit;

// Option bytes configuration module
pub mod option_bytes;


// ============================================================================
// Family-specific modules
// ============================================================================

// RA2 family (RA2E1, RA2E2, RA2L1)
#[cfg(any(ra2e1, ra2e2, ra2l1))]
mod ra2;
#[cfg(any(ra2e1, ra2e2, ra2l1))]
pub use ra2::Config;
#[cfg(any(ra2e1, ra2e2, ra2l1))]
pub(crate) use ra2::init;

// RA4 family (RA4M1, RA4M2, RA4M3, RA4E1, RA4E2, RA4W1)
#[cfg(any(ra4m1, ra4m2, ra4m3, ra4e1, ra4e2, ra4w1))]
mod ra4;
#[cfg(any(ra4m1, ra4m2, ra4m3, ra4e1, ra4e2, ra4w1))]
pub use ra4::Config;
#[cfg(any(ra4m1, ra4m2, ra4m3, ra4e1, ra4e2, ra4w1))]
pub(crate) use ra4::init;

// RA6 family (RA6M1, RA6M2, RA6M3, RA6M4, RA6M5, RA6E1, RA6E2, RA6T1, RA6T2)
#[cfg(any(ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6e1, ra6e2, ra6t1, ra6t2))]
mod ra6;
#[cfg(any(ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6e1, ra6e2, ra6t1, ra6t2))]
pub use ra6::Config;
#[cfg(any(ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6e1, ra6e2, ra6t1, ra6t2))]
pub(crate) use ra6::init;

// RA8 family (RA8M1, RA8D1, RA8E1, RA8T1)
#[cfg(any(ra8m1, ra8d1, ra8e1, ra8t1))]
mod ra8;
#[cfg(any(ra8m1, ra8d1, ra8e1, ra8t1))]
pub use ra8::Config;
#[cfg(any(ra8m1, ra8d1, ra8e1, ra8t1))]
pub(crate) use ra8::init;

// Fallback for unsupported families - use a basic config
#[cfg(not(any(
    ra2e1, ra2e2, ra2l1,
    ra4m1, ra4m2, ra4m3, ra4e1, ra4e2, ra4w1,
    ra6m1, ra6m2, ra6m3, ra6m4, ra6m5, ra6e1, ra6e2, ra6t1, ra6t2,
    ra8m1, ra8d1, ra8e1, ra8t1
)))]
compile_error!("No RA family detected. Make sure a chip feature is enabled (e.g., r7fa6e2ab).");

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
static mut CLOCK_FREQS: MaybeUninit<Clocks> = MaybeUninit::uninit();

/// Sets the clock frequencies
///
/// # Safety
/// Sets a mutable global. Must be called only once during initialization.
pub(crate) unsafe fn set_freqs(freqs: Clocks) {
    #[cfg(feature = "defmt")]
    defmt::debug!("rcc: clocks configured");
    CLOCK_FREQS = MaybeUninit::new(freqs);
}

/// Get the configured clock frequencies
///
/// # Safety
/// Reads a mutable global. Must only be called after `set_freqs`.
#[allow(dead_code)]
pub(crate) unsafe fn get_freqs() -> &'static Clocks {
    (*core::ptr::addr_of_mut!(CLOCK_FREQS)).assume_init_ref()
}

// ============================================================================
// Common Clock Types
// ============================================================================

/// Frequency in Hertz
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hertz(pub u32);

impl Hertz {
    pub const fn khz(khz: u32) -> Self {
        Self(khz * 1_000)
    }

    pub const fn mhz(mhz: u32) -> Self {
        Self(mhz * 1_000_000)
    }

    pub const fn hz(hz: u32) -> Self {
        Self(hz)
    }
}

/// Clock source selection for SCKSCR.CKSEL
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockSource {
    /// High-speed on-chip oscillator (HOCO)
    Hoco,
    /// Middle-speed on-chip oscillator (MOCO) - 8 MHz
    #[default]
    Moco,
    /// Low-speed on-chip oscillator (LOCO)
    Loco,
    /// Main clock oscillator (MOSC) - external crystal
    MainOsc,
    /// Sub-clock oscillator (SOSC) - 32.768 kHz crystal
    SubOsc,
    /// PLL output (not available on all families)
    Pll,
}

/// Clock divider values for SCKDIVCR
/// Divides the source clock by 2^n
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockDiv {
    #[default]
    Div1 = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
    Div16 = 4,
    Div32 = 5,
    Div64 = 6,
}

impl ClockDiv {
    pub const fn divisor(&self) -> u32 {
        1 << (*self as u32)
    }
}

/// HOCO frequency selection (chip-specific)
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HocoFreq {
    /// 24 MHz (RA2, RA4)
    Mhz24,
    /// 32 MHz (RA2)
    Mhz32,
    /// 48 MHz (RA2, RA4)
    #[default]
    Mhz48,
    /// 64 MHz (RA4)
    Mhz64,
    /// 16 MHz (RA6, RA8)
    Mhz16,
    /// 18 MHz (RA6, RA8)
    Mhz18,
    /// 20 MHz (RA6, RA8)
    Mhz20,
}

impl HocoFreq {
    pub const fn to_hz(&self) -> u32 {
        match self {
            HocoFreq::Mhz24 => 24_000_000,
            HocoFreq::Mhz32 => 32_000_000,
            HocoFreq::Mhz48 => 48_000_000,
            HocoFreq::Mhz64 => 64_000_000,
            HocoFreq::Mhz16 => 16_000_000,
            HocoFreq::Mhz18 => 18_000_000,
            HocoFreq::Mhz20 => 20_000_000,
        }
    }
}

/// Main oscillator (MOSC) mode
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MainOscMode {
    /// Crystal/ceramic resonator
    #[default]
    Crystal,
    /// External clock input
    ExternalClock,
}

/// Main oscillator configuration
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MainOscConfig {
    /// Oscillator frequency
    pub freq: Hertz,
    /// Oscillator mode
    pub mode: MainOscMode,
}

/// PLL input source
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PllSource {
    /// Main oscillator
    MainOsc,
    /// HOCO
    Hoco,
}

/// PLL input divider
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PllInputDiv {
    Div1 = 0,
    Div2 = 1,
    Div3 = 2,
    Div4 = 3,
}

/// PLL multiplier
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PllMul {
    Mul10_0 = 0x13,
    Mul10_5 = 0x14,
    Mul11_0 = 0x15,
    Mul11_5 = 0x16,
    Mul12_0 = 0x17,
    Mul12_5 = 0x18,
    Mul20_0 = 0x27,
    Mul20_5 = 0x28,
}

impl PllMul {
    /// Get the actual multiplier value (x2 to avoid floats)
    pub const fn value_x2(&self) -> u32 {
        match self {
            PllMul::Mul10_0 => 20,
            PllMul::Mul10_5 => 21,
            PllMul::Mul11_0 => 22,
            PllMul::Mul11_5 => 23,
            PllMul::Mul12_0 => 24,
            PllMul::Mul12_5 => 25,
            PllMul::Mul20_0 => 40,
            PllMul::Mul20_5 => 41, 
            // Matches will catch aliases automatically
        }
    }
}

/// PLL configuration
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PllConfig {
    /// PLL input source
    pub source: PllSource,
    /// PLL input divider (PLIDIV)
    pub input_div: PllInputDiv,
    /// PLL multiplier (PLLMUL)
    pub mul: PllMul,
}

// ============================================================================
// Frozen Clock Frequencies (output of configuration)
// ============================================================================

/// Configured clock frequencies
///
/// This struct contains all the clock frequencies after configuration.
/// It is "frozen" after init and cannot be changed at runtime.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Clocks {
    /// System clock (ICLK) frequency
    pub iclk: Hertz,
    /// Peripheral clock B (PCLKB) frequency
    pub pclkb: Hertz,
    /// Peripheral clock D (PCLKD) frequency  
    pub pclkd: Hertz,
    /// Flash clock (FCLK) frequency (if available)
    pub fclk: Option<Hertz>,
    /// Peripheral clock A (PCLKA) frequency (RA4/RA6/RA8)
    pub pclka: Option<Hertz>,
    /// Peripheral clock C (PCLKC) frequency (RA6/RA8)
    pub pclkc: Option<Hertz>,
    /// External bus clock (BCLK) frequency (RA6/RA8)
    pub bclk: Option<Hertz>,
}

impl Default for Clocks {
    fn default() -> Self {
        // Default to MOCO @ 8 MHz with no dividers
        Self {
            iclk: Hertz::mhz(8),
            pclkb: Hertz::mhz(8),
            pclkd: Hertz::mhz(8),
            fclk: None,
            pclka: None,
            pclkc: None,
            bclk: None,
        }
    }
}

// ============================================================================
// Clock Configuration
// ============================================================================

/// MOCO frequency (fixed at 8 MHz on all RA families)
pub const MOCO_FREQ: Hertz = Hertz(8_000_000);

/// LOCO frequency (fixed at 32.768 kHz)
pub const LOCO_FREQ: Hertz = Hertz(32_768);
