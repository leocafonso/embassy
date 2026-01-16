//! Reset and Clock Control (RCC) for Renesas RA
//!
//! This module handles clock configuration for RA MCUs via the SYSC/SYSTEM peripheral.
//! Different RA families have different clock sources and dividers available.

#![allow(missing_docs)]

use core::mem::MaybeUninit;

use crate::pac::peripherals;

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
    /// 48 MHz (RA2, RA4, RA6)
    #[default]
    Mhz48,
    /// 64 MHz (RA4, RA6)
    Mhz64,
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

/// Clock configuration
///
/// This is a unified configuration struct that works across all RA families.
/// Not all fields are used on all families - unused fields are ignored.
#[non_exhaustive]
#[derive(Clone, Copy, Default)]
pub struct Config {
    /// Clock source selection
    pub source: ClockSource,
    
    /// HOCO frequency (if HOCO is used as source)
    pub hoco_freq: HocoFreq,
    
    /// Main oscillator configuration (if MOSC is used)
    pub main_osc: Option<MainOscConfig>,
    
    /// System clock (ICLK) divider
    pub iclk_div: ClockDiv,
    
    /// Peripheral clock A (PCLKA) divider (RA4/RA6/RA8 only)
    pub pclka_div: ClockDiv,
    
    /// Peripheral clock B (PCLKB) divider
    pub pclkb_div: ClockDiv,
    
    /// Peripheral clock C (PCLKC) divider (RA6/RA8 only)
    pub pclkc_div: ClockDiv,
    
    /// Peripheral clock D (PCLKD) divider
    pub pclkd_div: ClockDiv,
    
    /// Flash clock (FCLK) divider (RA4/RA6/RA8 only)
    pub fclk_div: ClockDiv,
    
    /// External bus clock (BCLK) divider (RA6/RA8 only)
    pub bclk_div: ClockDiv,
}

impl Config {
    /// Create configuration using HOCO at specified frequency
    pub const fn hoco(freq: HocoFreq) -> Self {
        Self {
            source: ClockSource::Hoco,
            hoco_freq: freq,
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkc_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
            bclk_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using MOCO (8 MHz)
    pub const fn moco() -> Self {
        Self {
            source: ClockSource::Moco,
            hoco_freq: HocoFreq::Mhz48,
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkc_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
            bclk_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using external main oscillator
    pub const fn main_osc(config: MainOscConfig) -> Self {
        Self {
            source: ClockSource::MainOsc,
            hoco_freq: HocoFreq::Mhz48,
            main_osc: Some(config),
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkc_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
            bclk_div: ClockDiv::Div1,
        }
    }
    
    /// Set ICLK divider
    pub const fn iclk_div(mut self, div: ClockDiv) -> Self {
        self.iclk_div = div;
        self
    }
    
    /// Set PCLKB divider
    pub const fn pclkb_div(mut self, div: ClockDiv) -> Self {
        self.pclkb_div = div;
        self
    }
    
    /// Set PCLKD divider
    pub const fn pclkd_div(mut self, div: ClockDiv) -> Self {
        self.pclkd_div = div;
        self
    }
}

/// Initialize clocks with the given configuration
pub(crate) fn init(config: Config) -> Clocks {
    // Calculate source frequency
    let source_freq = match config.source {
        ClockSource::Hoco => Hertz(config.hoco_freq.to_hz()),
        ClockSource::Moco => MOCO_FREQ,
        ClockSource::Loco => LOCO_FREQ,
        ClockSource::MainOsc => {
            config.main_osc.expect("MainOsc config required when using MainOsc source").freq
        }
        ClockSource::SubOsc => Hertz(32_768),
        ClockSource::Pll => {
            // TODO: Implement PLL calculation
            panic!("PLL configuration not yet implemented");
        }
    };
    
    // Calculate output frequencies
    let iclk = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    
    // Configure SCKSCR
    let cksel: u8 = match config.source {
        ClockSource::Hoco => 0b000,
        ClockSource::Moco => 0b001,
        ClockSource::Loco => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc => 0b100,
        ClockSource::Pll => 0b101,
    };
    
    // Write to registers using modify() which gives access to typed fields
    let sysc = unsafe { peripherals::SYSC::steal() };
    
    sysc.sckdivcr().modify(|w| {
        w.set_pckd((config.pclkd_div as u8).into());
        w.set_pckb((config.pclkb_div as u8).into());
        w.set_ick((config.iclk_div as u8).into());
    });
    
    sysc.sckscr().modify(|w| {
        w.set_cksel(cksel.into());
    });
    
    Clocks {
        iclk,
        pclkb,
        pclkd,
        fclk: None,
        pclka: None,
        pclkc: None,
        bclk: None,
    }
}
