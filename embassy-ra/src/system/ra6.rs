//! Clock configuration for RA6 family (RA6M1-M5, RA6E1-E2, RA6T1-T2)
//!
//! RA6 family characteristics:
//! - Max ICLK: 200 MHz (varies by sub-family)
//! - Clock sources: HOCO (16/18/20 MHz), MOCO (8 MHz), LOCO, MOSC, SOSC, PLL, PLL2
//! - PLL with configurable multiplier and dividers
//! - Dividers: ICLK, PCLKA, PCLKB, PCLKC, PCLKD, FCLK, BCLK
//!
//! Reference: RA6M5 Hardware Manual, Section 8 (Clocks)

use defmt::debug;

use ra_metapac::sysc::Sysc;
use super::{Clocks, ClockDiv, ClockSource, Hertz, HocoFreq, MainOscConfig, PllConfig, PllSource, PllInputDiv, MOCO_FREQ, LOCO_FREQ};

/// PLL2 configuration (alias for PLL config structure)
pub type Pll2Config = PllConfig;

// Use direct PAC access
fn sysc() -> Sysc {
    ra_metapac::SYSC
}

/// Maximum ICLK frequency for RA6 family (varies by sub-family)
/// RA6E2: 200 MHz, RA6M5: 200 MHz, others: 120-200 MHz
pub const MAX_ICLK_FREQ: Hertz = Hertz(200_000_000);

/// Maximum PCLKA frequency
pub const MAX_PCLKA_FREQ: Hertz = Hertz(100_000_000);

/// Maximum PCLKB frequency
pub const MAX_PCLKB_FREQ: Hertz = Hertz(50_000_000);

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
            pclka_div: ClockDiv::Div2,
            pclkb_div: ClockDiv::Div2,
            pclkc_div: ClockDiv::Div2,
            pclkd_div: ClockDiv::Div2,
            fclk_div: ClockDiv::Div4,
            bclk_div: ClockDiv::Div2,
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
            pclka_div: ClockDiv::Div2,
            pclkb_div: ClockDiv::Div2,
            pclkc_div: ClockDiv::Div2,
            pclkd_div: ClockDiv::Div2,
            fclk_div: ClockDiv::Div4,
            bclk_div: ClockDiv::Div2,
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
    let sysc = sysc();
    
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

            Hertz(pll_input.0 / input_div)  // Assuming output divider of /2 for PLLCLK
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
    
    debug!("Clock frequencies: ICLK={} MHz, PCLKA={} MHz, PCLKB={} MHz, PCLKC={} MHz, PCLKD={} MHz, FCLK={} MHz, BCLK={} MHz",
        iclk.0 / 1_000_000,
        pclka.0 / 1_000_000,
        pclkb.0 / 1_000_000,
        pclkc.0 / 1_000_000,
        pclkd.0 / 1_000_000,
        fclk.0 / 1_000_000,
        bclk.0 / 1_000_000,
    );

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

    // Configure HOCO frequency if HOCO is selected or used by PLL
    if matches!(config.source, ClockSource::Hoco) || 
       (matches!(config.source, ClockSource::Pll) && matches!(config.pll.unwrap().source, PllSource::Hoco)) {
        
        // 00: 16 MHz, 01: 18 MHz, 10: 20 MHz
        let hcfrq: u8 = match config.hoco_freq {
            HocoFreq::Mhz16 => 0b00,
            HocoFreq::Mhz18 => 0b01,
            HocoFreq::Mhz20 => 0b10,
            _ => 0b10, // Default to 20MHz for unsupported values
        };
        
        // Ensure HOCO is operating
        sysc.hococr().modify(|w| w.set_hcstp(false));
         
        // Set frequency
        // Note: Writing to HOCOCR2 while HOCO is running is allowed on RA6
        // HOCOCR2 is at offset +1 from HOCOCR (0x4001E037 vs 0x4001E036 on RA6M5)
        // Since ra-metapac is missing hococr2 on some variants, we use raw pointer arithmetic.
        unsafe {
            let hococr_ptr = sysc.hococr().as_ptr() as *mut u8;
            hococr_ptr.add(1).write_volatile(hcfrq);
        }
        
        // TODO: Wait for stabilization if it was stopped
    }

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
