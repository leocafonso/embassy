//! Clock configuration for RA2 family (RA2E1, RA2E2, RA2L1)
//!
//! RA2 family characteristics:
//! - Max ICLK: 48 MHz
//! - Clock sources: HOCO (24/32/48 MHz), MOCO (8 MHz), LOCO (32.768 kHz), MOSC, SOSC
//! - No PLL
//! - Dividers: ICLK, PCLKB, PCLKD
//!
//! Reference: RA2E1 Hardware Manual, Section 8 (Clocks)

use crate::pac::peripherals::SYSC;
use super::{Clocks, ClockDiv, ClockSource, Hertz, HocoFreq, MainOscConfig};

/// MOCO frequency (fixed at 8 MHz)
pub const MOCO_FREQ: Hertz = Hertz(8_000_000);

/// LOCO frequency (fixed at 32.768 kHz)
pub const LOCO_FREQ: Hertz = Hertz(32_768);

/// Maximum ICLK frequency for RA2 family
pub const MAX_ICLK_FREQ: Hertz = Hertz(48_000_000);

/// Clock configuration for RA2 family
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source selection
    pub source: ClockSource,
    
    /// HOCO frequency (if HOCO is used as source)
    pub hoco_freq: HocoFreq,
    
    /// Main oscillator configuration (if MOSC is used)
    pub main_osc: Option<MainOscConfig>,
    
    /// System clock (ICLK) divider
    pub iclk_div: ClockDiv,
    
    /// Peripheral clock B (PCLKB) divider
    /// Used by: SCI, SPI, I2C, GPT, etc.
    pub pclkb_div: ClockDiv,
    
    /// Peripheral clock D (PCLKD) divider
    /// Used by: ADC, DAC
    pub pclkd_div: ClockDiv,
}

impl Default for Config {
    fn default() -> Self {
        // Default: MOCO @ 8 MHz, no dividers
        // This is the reset state
        Self {
            source: ClockSource::Moco,
            hoco_freq: HocoFreq::Mhz48,
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
        }
    }
}

impl Config {
    /// Create configuration using HOCO at specified frequency
    pub const fn hoco(freq: HocoFreq) -> Self {
        Self {
            source: ClockSource::Hoco,
            hoco_freq: freq,
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using MOCO (8 MHz)
    pub const fn moco() -> Self {
        Self {
            source: ClockSource::Moco,
            hoco_freq: HocoFreq::Mhz48, // unused
            main_osc: None,
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using external main oscillator
    pub const fn main_osc(config: MainOscConfig) -> Self {
        Self {
            source: ClockSource::MainOsc,
            hoco_freq: HocoFreq::Mhz48, // unused
            main_osc: Some(config),
            iclk_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
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
///
/// # Safety
/// This function should only be called once during system initialization.
pub(crate) fn init(config: Config) -> Clocks {
    let sysc = unsafe { SYSC::steal() };
    
    // Calculate source frequency
    let source_freq = match config.source {
        ClockSource::Hoco => {
            // Enable HOCO if not already enabled
            // TODO: Configure HOCO frequency via HOCOWTCR if needed
            Hertz(config.hoco_freq.to_hz())
        }
        ClockSource::Moco => MOCO_FREQ,
        ClockSource::Loco => LOCO_FREQ,
        ClockSource::MainOsc => {
            config.main_osc.expect("MainOsc config required").freq
        }
        ClockSource::SubOsc => Hertz(32_768),
        ClockSource::Pll => panic!("RA2 family does not have PLL"),
    };
    
    // Calculate output frequencies
    let iclk = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    
    // Validate frequencies
    assert!(iclk.0 <= MAX_ICLK_FREQ.0, "ICLK exceeds maximum frequency");
    
    // Configure SCKDIVCR (System Clock Division Control Register)
    // Bits [2:0]   - PCKD (Peripheral Clock D)
    // Bits [10:8]  - PCKB (Peripheral Clock B)  
    // Bits [26:24] - ICK (System Clock)
    let sckdivcr = (config.pclkd_div as u32)
        | ((config.pclkb_div as u32) << 8)
        | ((config.iclk_div as u32) << 24);
    
    sysc.sckdivcr().write_value(sckdivcr);
    
    // Configure SCKSCR (System Clock Source Control Register)
    let cksel = match config.source {
        ClockSource::Hoco => 0b000,
        ClockSource::Moco => 0b001,
        ClockSource::Loco => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc => 0b100,
        ClockSource::Pll => unreachable!(),
    };
    
    // TODO: Add proper clock switching sequence with stabilization wait
    sysc.sckscr().write_value(cksel);
    
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
