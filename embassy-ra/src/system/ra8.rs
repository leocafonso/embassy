//! Clock configuration for RA8 family (RA8M1, RA8D1, RA8T1)
//!
//! RA8 family characteristics:
//! - Max ICLK: 480 MHz
//! - Clock sources: HOCO (16/18/20 MHz), MOCO (8 MHz), LOCO, MOSC, SOSC, PLL, PLL2
//! - Advanced PLL with wide frequency range
//! - Dividers: ICLK, PCLKA, PCLKB, PCLKC, PCLKD, FCLK, BCLK
//! - Additional clocks: CANFDCLK, SCICLK, SPICLK, etc.
//!
//! Reference: RA8M1 Hardware Manual, Section 8 (Clocks)

use crate::pac;
use super::{Clocks, ClockDiv, ClockSource, Hertz, HocoFreq, MainOscConfig};

// Use direct PAC access
fn sysc() -> pac::sysc::Sysc {
    pac::SYSC
}

/// MOCO frequency (fixed at 8 MHz)
pub const MOCO_FREQ: Hertz = Hertz(8_000_000);

/// LOCO frequency (fixed at 32.768 kHz)
pub const LOCO_FREQ: Hertz = Hertz(32_768);

/// Maximum ICLK frequency for RA8
pub const MAX_ICLK_FREQ: Hertz = Hertz(480_000_000);

// TODO: Add PLL configuration structures for RA8

/// Clock configuration for RA8 family
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source selection
    pub source: ClockSource,
    
    /// HOCO frequency (if HOCO is used)
    pub hoco_freq: HocoFreq,
    
    /// Main oscillator configuration
    pub main_osc: Option<MainOscConfig>,
    
    // TODO: Add PLL configuration
    
    /// System clock (ICLK) divider
    pub iclk_div: ClockDiv,
    
    /// Peripheral clock A divider
    pub pclka_div: ClockDiv,
    
    /// Peripheral clock B divider
    pub pclkb_div: ClockDiv,
    
    /// Peripheral clock C divider
    pub pclkc_div: ClockDiv,
    
    /// Peripheral clock D divider
    pub pclkd_div: ClockDiv,
    
    /// Flash clock divider
    pub fclk_div: ClockDiv,
    
    /// External bus clock divider
    pub bclk_div: ClockDiv,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            source: ClockSource::Hoco,
            hoco_freq: HocoFreq::Mhz20,
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
}

impl Config {
    /// Create configuration using HOCO
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
}

/// Initialize clocks with the given configuration
pub(crate) fn init(config: Config) -> Clocks {
    let sysc = sysc();
    
    // Calculate source frequency
    let source_freq = match config.source {
        ClockSource::Hoco => Hertz(config.hoco_freq.to_hz()),
        ClockSource::Moco => MOCO_FREQ,
        ClockSource::Loco => LOCO_FREQ,
        ClockSource::MainOsc => config.main_osc.expect("MainOsc required").freq,
        ClockSource::SubOsc => Hertz(32_768),
        ClockSource::Pll => todo!("PLL not yet implemented for RA8"),
    };
    
    // Calculate output frequencies
    let iclk = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclka = Hertz(source_freq.0 / config.pclka_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkc = Hertz(source_freq.0 / config.pclkc_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    let fclk = Hertz(source_freq.0 / config.fclk_div.divisor());
    let bclk = Hertz(source_freq.0 / config.bclk_div.divisor());
    
    // Unlock register protection for clock configuration
    sysc.prcr().write(|w| {
        w.set_prkey(0xA5.into());  // Write key
        w.set_prc0(true);   // Enable writing to clock registers
    });

    // Configure SCKDIVCR (System Clock Division Control Register)
    sysc.sckdivcr().modify(|w| {
        w.set_pckd((config.pclkd_div as u8).into());
        w.set_pckc((config.pclkc_div as u8).into());
        w.set_pckb((config.pclkb_div as u8).into());
        w.set_pcka((config.pclka_div as u8).into());
        w.set_ick((config.iclk_div as u8).into());
        w.set_fck((config.fclk_div as u8).into());
    });
    
    // Configure SCKSCR
    let cksel = match config.source {
        ClockSource::Hoco => 0b000,
        ClockSource::Moco => 0b001,
        ClockSource::Loco => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc => 0b100,
        ClockSource::Pll => 0b101,
    };
    
    sysc.sckscr().modify(|w| {
        w.set_cksel(cksel.into());
    });

    // Lock register protection
    sysc.prcr().write(|w| {
        w.set_prkey(0xA5.into());
        w.set_prc0(false);
    });
    
    Clocks {
        iclk,
        pclkb,
        pclkd,
        fclk: Some(fclk),
        pclka: Some(pclka),
        pclkc: Some(pclkc),
        bclk: Some(bclk),
    }
}
