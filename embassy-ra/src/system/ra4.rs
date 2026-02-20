//! Clock configuration for RA4 family (RA4M1, RA4M2, RA4M3, RA4E1, RA4E2, RA4T1)
//!
//! RA4 family characteristics:
//! - Max ICLK: 100 MHz (varies by sub-family)
//! - Clock sources: HOCO (24/32/48/64 MHz), MOCO (8 MHz), LOCO, MOSC, SOSC, PLL
//! - PLL available with configurable multiplier and dividers
//! - Dividers: ICLK, PCLKA, PCLKB, PCLKD, FCLK
//!
//! Reference: RA4M1 Hardware Manual, Section 8 (Clocks)

use crate::peripherals::SYSTEM;
use super::{Clocks, ClockDiv, ClockSource, Hertz, HocoFreq, MainOscConfig, PllConfig, PllSource, PllInputDiv, PllMul, MOCO_FREQ, LOCO_FREQ};

/// Maximum ICLK frequency for RA4M1
pub const MAX_ICLK_FREQ: Hertz = Hertz(48_000_000);

/// Clock configuration for RA4 family
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source selection
    pub source: ClockSource,
    
    /// HOCO frequency (if HOCO is used)
    pub hoco_freq: HocoFreq,
    
    /// Main oscillator configuration (if MOSC is used)
    pub main_osc: Option<MainOscConfig>,
    
    /// PLL configuration (if PLL is used)
    pub pll: Option<PllConfig>,
    
    /// System clock (ICLK) divider
    pub iclk_div: ClockDiv,
    
    /// Peripheral clock A (PCLKA) divider
    /// Used by: ETHERC, EDMAC, USBFS
    pub pclka_div: ClockDiv,
    
    /// Peripheral clock B (PCLKB) divider
    /// Used by: SCI, SPI, I2C, GPT, etc.
    pub pclkb_div: ClockDiv,
    
    /// Peripheral clock D (PCLKD) divider
    /// Used by: ADC, DAC
    pub pclkd_div: ClockDiv,
    
    /// Flash clock (FCLK) divider
    pub fclk_div: ClockDiv,
}

impl Default for Config {
    fn default() -> Self {
        // Default: MOCO @ 8 MHz, no dividers
        Self {
            source: ClockSource::Moco,
            hoco_freq: HocoFreq::Mhz48,
            main_osc: None,
            pll: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
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
            pll: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using MOCO (8 MHz)
    pub const fn moco() -> Self {
        Self {
            source: ClockSource::Moco,
            hoco_freq: HocoFreq::Mhz48,
            main_osc: None,
            pll: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using PLL
    pub const fn pll(pll_config: PllConfig, main_osc: Option<MainOscConfig>) -> Self {
        Self {
            source: ClockSource::Pll,
            hoco_freq: HocoFreq::Mhz48,
            main_osc,
            pll: Some(pll_config),
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
        }
    }
    
    /// Set ICLK divider
    pub const fn iclk_div(mut self, div: ClockDiv) -> Self {
        self.iclk_div = div;
        self
    }
    
    /// Set PCLKA divider
    pub const fn pclka_div(mut self, div: ClockDiv) -> Self {
        self.pclka_div = div;
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
    
    /// Set FCLK divider
    pub const fn fclk_div(mut self, div: ClockDiv) -> Self {
        self.fclk_div = div;
        self
    }
}

/// Initialize clocks with the given configuration
///
/// # Safety
/// This function should only be called once during system initialization.
pub(crate) fn init(config: Config) -> Clocks {
    let system = unsafe { SYSTEM::steal() };
    
    // Calculate source frequency
    let source_freq = match config.source {
        ClockSource::Hoco => Hertz(config.hoco_freq.to_hz()),
        ClockSource::Moco => MOCO_FREQ,
        ClockSource::Loco => LOCO_FREQ,
        ClockSource::MainOsc => {
            config.main_osc.expect("MainOsc config required").freq
        }
        ClockSource::SubOsc => Hertz(32_768),
        ClockSource::Pll => {
            let pll = config.pll.expect("PLL config required");
            let pll_input = match pll.source {
                PllSource::MainOsc => config.main_osc.expect("MainOsc required for PLL").freq,
                PllSource::Hoco => Hertz(config.hoco_freq.to_hz()),
            };
            // TODO: Calculate actual PLL output frequency
            // PLL output = (input / PLIDIV) * PLLMUL
            pll_input // Placeholder
        }
    };
    
    // Calculate output frequencies
    let iclk = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclka = Hertz(source_freq.0 / config.pclka_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    let fclk = Hertz(source_freq.0 / config.fclk_div.divisor());
    
    // Validate frequencies
    assert!(iclk.0 <= MAX_ICLK_FREQ.0, "ICLK exceeds maximum frequency");
    
    // Configure SCKDIVCR (System Clock Division Control Register)
    // Bits [2:0]   - PCKD (Peripheral Clock D)
    // Bits [10:8]  - PCKB (Peripheral Clock B)
    // Bits [14:12] - PCKA (Peripheral Clock A)
    // Bits [26:24] - ICK (System Clock)
    // Bits [30:28] - FCK (Flash Clock)
    let sckdivcr = (config.pclkd_div as u32)
        | ((config.pclkb_div as u32) << 8)
        | ((config.pclka_div as u32) << 12)
        | ((config.iclk_div as u32) << 24)
        | ((config.fclk_div as u32) << 28);
    
    system.sckdivcr().write_value(sckdivcr);
    
    // Configure SCKSCR (System Clock Source Control Register)
    let cksel = match config.source {
        ClockSource::Hoco => 0b000,
        ClockSource::Moco => 0b001,
        ClockSource::Loco => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc => 0b100,
        ClockSource::Pll => 0b101,
    };
    
    // TODO: Add proper clock switching sequence with stabilization wait
    system.sckscr().write_value(cksel);
    
    Clocks {
        iclk,
        pclkb,
        pclkd,
        fclk: Some(fclk),
        pclka: Some(pclka),
        pclkc: None,
        bclk: None,
    }
}
