//! Clock configuration for RA6 family (RA6M1-M5, RA6E1-E2, RA6T1-T2)
//!
//! RA6 family characteristics:
//! - Max ICLK: 200 MHz (varies by sub-family)
//! - Clock sources: HOCO (16/18/20 MHz), MOCO (8 MHz), LOCO, MOSC, SOSC, PLL, PLL2
//! - PLL with configurable multiplier and dividers
//! - Dividers: ICLK, PCLKA, PCLKB, PCLKC, PCLKD, FCLK, BCLK
//!
//! Reference: RA6M5 Hardware Manual, Section 8 (Clocks)

use crate::pac::peripherals::SYSC;
use super::{Clocks, ClockDiv, ClockSource, Hertz, HocoFreq, MainOscConfig};

/// MOCO frequency (fixed at 8 MHz)
pub const MOCO_FREQ: Hertz = Hertz(8_000_000);

/// LOCO frequency (fixed at 32.768 kHz)
pub const LOCO_FREQ: Hertz = Hertz(32_768);

/// Maximum ICLK frequency for RA6 family (varies by sub-family)
/// RA6E2: 200 MHz, RA6M5: 200 MHz, others: 120-200 MHz
pub const MAX_ICLK_FREQ: Hertz = Hertz(200_000_000);

/// Maximum PCLKA frequency
pub const MAX_PCLKA_FREQ: Hertz = Hertz(100_000_000);

/// Maximum PCLKB frequency
pub const MAX_PCLKB_FREQ: Hertz = Hertz(50_000_000);

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
    /// PLL output divider for PLLCLK
    pub output_div: PllOutputDiv,
}

/// PLL2 configuration (for USB, etc.)
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Pll2Config {
    /// PLL2 input source
    pub source: PllSource,
    /// PLL2 input divider
    pub input_div: PllInputDiv,
    /// PLL2 multiplier
    pub mul: PllMul,
    /// PLL2 output divider
    pub output_div: PllOutputDiv,
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

/// PLL multiplier (x10.0 to x30.0 in 0.5 steps)
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
    // TODO: Add all multiplier values
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
        }
    }
}

/// PLL output divider
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PllOutputDiv {
    Div2 = 0,
    Div3 = 1,
    Div4 = 2,
}

impl PllOutputDiv {
    pub const fn divisor(&self) -> u32 {
        match self {
            PllOutputDiv::Div2 => 2,
            PllOutputDiv::Div3 => 3,
            PllOutputDiv::Div4 => 4,
        }
    }
}

/// Clock configuration for RA6 family
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Clock source selection
    pub source: ClockSource,
    
    /// HOCO frequency (if HOCO is used)
    pub hoco_freq: HocoFreq,
    
    /// Main oscillator configuration (if MOSC is used)
    pub main_osc: Option<MainOscConfig>,
    
    /// PLL configuration (if PLL is used as source)
    pub pll: Option<PllConfig>,
    
    /// PLL2 configuration (for USB, etc.)
    pub pll2: Option<Pll2Config>,
    
    /// System clock (ICLK) divider
    pub iclk_div: ClockDiv,
    
    /// Peripheral clock A (PCLKA) divider
    pub pclka_div: ClockDiv,
    
    /// Peripheral clock B (PCLKB) divider
    pub pclkb_div: ClockDiv,
    
    /// Peripheral clock C (PCLKC) divider
    pub pclkc_div: ClockDiv,
    
    /// Peripheral clock D (PCLKD) divider
    pub pclkd_div: ClockDiv,
    
    /// Flash clock (FCLK) divider
    pub fclk_div: ClockDiv,
    
    /// External bus clock (BCLK) divider
    pub bclk_div: ClockDiv,
}

impl Default for Config {
    fn default() -> Self {
        // Default: HOCO @ 20 MHz, no dividers
        Self {
            source: ClockSource::Hoco,
            hoco_freq: HocoFreq::Mhz20,
            main_osc: None,
            pll: None,
            pll2: None,
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
    /// Create configuration using HOCO at specified frequency
    pub const fn hoco(freq: HocoFreq) -> Self {
        Self {
            source: ClockSource::Hoco,
            hoco_freq: freq,
            main_osc: None,
            pll: None,
            pll2: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div1,
            pclkb_div: ClockDiv::Div1,
            pclkc_div: ClockDiv::Div1,
            pclkd_div: ClockDiv::Div1,
            fclk_div: ClockDiv::Div1,
            bclk_div: ClockDiv::Div1,
        }
    }
    
    /// Create configuration using PLL with MOSC input
    /// 
    /// Example for 200 MHz from 24 MHz crystal:
    /// - MOSC = 24 MHz
    /// - PLIDIV = /2 → 12 MHz
    /// - PLLMUL = x20 → 240 MHz VCO
    /// - PLODIV = /2 → 120 MHz PLLCLK (then ICLK divider)
    pub const fn pll_from_mosc(main_osc: MainOscConfig, pll: PllConfig) -> Self {
        Self {
            source: ClockSource::Pll,
            hoco_freq: HocoFreq::Mhz20,
            main_osc: Some(main_osc),
            pll: Some(pll),
            pll2: None,
            iclk_div: ClockDiv::Div1,
            pclka_div: ClockDiv::Div2,
            pclkb_div: ClockDiv::Div4,
            pclkc_div: ClockDiv::Div4,
            pclkd_div: ClockDiv::Div2,
            fclk_div: ClockDiv::Div4,
            bclk_div: ClockDiv::Div2,
        }
    }
    
    /// Set ICLK divider
    pub const fn iclk_div(mut self, div: ClockDiv) -> Self {
        self.iclk_div = div;
        self
    }
    
    /// Set all peripheral clock dividers at once
    pub const fn pclk_divs(
        mut self,
        pclka: ClockDiv,
        pclkb: ClockDiv,
        pclkc: ClockDiv,
        pclkd: ClockDiv,
    ) -> Self {
        self.pclka_div = pclka;
        self.pclkb_div = pclkb;
        self.pclkc_div = pclkc;
        self.pclkd_div = pclkd;
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
            
            // PLL output = (input / PLIDIV) * PLLMUL / PLODIV
            let input_div = match pll.input_div {
                PllInputDiv::Div1 => 1,
                PllInputDiv::Div2 => 2,
                PllInputDiv::Div3 => 3,
                PllInputDiv::Div4 => 4,
            };
            
            let vco_freq = (pll_input.0 / input_div) * pll.mul.value_x2() / 2;
            Hertz(vco_freq / pll.output_div.divisor())
        }
    };
    
    // Calculate output frequencies
    let iclk = Hertz(source_freq.0 / config.iclk_div.divisor());
    let pclka = Hertz(source_freq.0 / config.pclka_div.divisor());
    let pclkb = Hertz(source_freq.0 / config.pclkb_div.divisor());
    let pclkc = Hertz(source_freq.0 / config.pclkc_div.divisor());
    let pclkd = Hertz(source_freq.0 / config.pclkd_div.divisor());
    let fclk = Hertz(source_freq.0 / config.fclk_div.divisor());
    let bclk = Hertz(source_freq.0 / config.bclk_div.divisor());
    
    // Validate frequencies
    assert!(iclk.0 <= MAX_ICLK_FREQ.0, "ICLK exceeds maximum frequency");
    assert!(pclka.0 <= MAX_PCLKA_FREQ.0, "PCLKA exceeds maximum frequency");
    assert!(pclkb.0 <= MAX_PCLKB_FREQ.0, "PCLKB exceeds maximum frequency");
    
    // TODO: Configure flash wait states based on ICLK frequency
    
    // Unlock register protection for clock configuration
    sysc.prcr().modify(|w| {
        w.set_prkey(0xA5);  // Write key
        w.set_prc0(true);   // Enable writing to clock registers
    });

    // Configure SCKDIVCR (System Clock Division Control Register)
    sysc.sckdivcr().modify(|w| {
        w.set_pckd((config.pclkd_div as u8).into());
        w.set_pckc((config.pclkc_div as u8).into());
        w.set_pckb((config.pclkb_div as u8).into());
        w.set_pcka((config.pclka_div as u8).into());
        // Note: BCK (external bus clock) not available on all RA6 variants (e.g., RA6E2)
        // w.set_bck((config.bclk_div as u8).into());
        w.set_ick((config.iclk_div as u8).into());
        w.set_fck((config.fclk_div as u8).into());
    });
    
    // Configure SCKSCR (System Clock Source Control Register)
    let cksel: u8 = match config.source {
        ClockSource::Hoco => 0b000,
        ClockSource::Moco => 0b001,
        ClockSource::Loco => 0b010,
        ClockSource::MainOsc => 0b011,
        ClockSource::SubOsc => 0b100,
        ClockSource::Pll => 0b101,
    };
    
    // TODO: Add proper clock switching sequence:
    // 1. Enable target oscillator/PLL
    // 2. Wait for stabilization
    // 3. Switch clock source
    // 4. Disable unused oscillators (optional, for power saving)
    
    sysc.sckscr().modify(|w| {
        w.set_cksel(cksel.into());
    });
    
    // Lock register protection
    sysc.prcr().modify(|w| {
        w.set_prkey(0xA5);
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
